use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use super::state::{OllamaModel, OllamaSnapshot, OllamaStatus};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct OllamaProber {
    client: Client,
    host: String,
}

impl OllamaProber {
    pub fn new(host: String) -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("default reqwest client should always build");
        Self { client, host }
    }

    pub async fn probe(&self) -> OllamaSnapshot {
        let version = match self.fetch_version().await {
            Some(v) => v,
            None => {
                return OllamaSnapshot {
                    status: OllamaStatus::Unreachable,
                    models: vec![],
                }
            }
        };

        let models = self.fetch_loaded_models().await.unwrap_or_default();

        OllamaSnapshot {
            status: OllamaStatus::Reachable { version },
            models,
        }
    }

    async fn fetch_version(&self) -> Option<String> {
        #[derive(Deserialize)]
        struct VersionResp {
            version: String,
        }

        let url = format!("{}/api/version", self.host.trim_end_matches('/'));
        self.client
            .get(&url)
            .send()
            .await
            .ok()?
            .json::<VersionResp>()
            .await
            .ok()
            .map(|v| v.version)
    }

    async fn fetch_loaded_models(&self) -> Option<Vec<OllamaModel>> {
        #[derive(Deserialize)]
        struct PsResp {
            models: Vec<PsModel>,
        }
        #[derive(Deserialize)]
        struct PsModel {
            name: String,
            size_vram: u64,
            #[serde(default)]
            expires_at: Option<String>,
        }

        let url = format!("{}/api/ps", self.host.trim_end_matches('/'));
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .ok()?
            .json::<PsResp>()
            .await
            .ok()?;

        let any_has_expires_at = resp.models.iter().any(|m| m.expires_at.is_some());
        Some(
            resp.models
                .into_iter()
                .map(|m| {
                    let (is_active, dot_known) =
                        classify_active(m.expires_at.as_deref(), any_has_expires_at, Utc::now());
                    OllamaModel {
                        name: m.name,
                        size_vram_bytes: m.size_vram,
                        is_active,
                        dot_known,
                    }
                })
                .collect(),
        )
    }
}

/// Heuristic for the activity dot — see spec §"Active vs idle determination."
///
/// Returns `(is_active, dot_known)`.
fn classify_active(
    expires_at: Option<&str>,
    any_has_expires_at: bool,
    now: DateTime<Utc>,
) -> (bool, bool) {
    if !any_has_expires_at {
        // Whole response missing expires_at → API drift / older Ollama.
        // Hide the dot column for this snapshot.
        return (false, false);
    }
    let Some(s) = expires_at else {
        // This row omitted the field but others have it → assume in-flight request.
        return (true, true);
    };
    match DateTime::parse_from_rfc3339(s) {
        Ok(t) => {
            let active = t.with_timezone(&Utc) <= now;
            (active, true)
        }
        Err(_) => {
            // Malformed timestamp → treat as idle but keep the dot visible.
            (false, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 26, 12, 0, 0).unwrap()
    }

    #[test]
    fn classify_when_no_model_has_expires_at_drops_dot() {
        let (active, known) = classify_active(None, false, now());
        assert!(!active);
        assert!(!known);
    }

    #[test]
    fn classify_when_field_missing_on_this_row_only_marks_active() {
        let (active, known) = classify_active(None, true, now());
        assert!(active);
        assert!(known);
    }

    #[test]
    fn classify_with_future_expires_at_is_idle() {
        let future = "2026-04-26T12:05:00Z";
        let (active, known) = classify_active(Some(future), true, now());
        assert!(!active);
        assert!(known);
    }

    #[test]
    fn classify_with_past_expires_at_is_active() {
        let past = "2026-04-26T11:55:00Z";
        let (active, known) = classify_active(Some(past), true, now());
        assert!(active);
        assert!(known);
    }

    #[test]
    fn classify_with_malformed_timestamp_is_idle_with_dot() {
        let (active, known) = classify_active(Some("not-a-timestamp"), true, now());
        assert!(!active);
        assert!(known);
    }

    #[tokio::test]
    async fn probe_reachable_with_models() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "version": "0.5.7"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/ps"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "models": [
                    {
                        "name": "llama3.1:8b",
                        "size_vram": 4_500_000_000_u64,
                        "expires_at": "2030-01-01T00:00:00Z"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let prober = OllamaProber::new(server.uri());

        let snap = prober.probe().await;

        assert!(matches!(snap.status, OllamaStatus::Reachable { .. }));
        assert_eq!(snap.models.len(), 1);
        assert_eq!(snap.models[0].name, "llama3.1:8b");
        assert!(!snap.models[0].is_active);
        assert!(snap.models[0].dot_known);
    }

    #[tokio::test]
    async fn probe_version_endpoint_unreachable() {
        let prober = OllamaProber::new("http://127.0.0.1:1".into());

        let snap = prober.probe().await;

        assert_eq!(snap.status, OllamaStatus::Unreachable);
        assert!(snap.models.is_empty());
    }

    #[tokio::test]
    async fn probe_reachable_with_no_models() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "version": "0.5.7"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/ps"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "models": []
            })))
            .mount(&server)
            .await;

        let prober = OllamaProber::new(server.uri());

        let snap = prober.probe().await;

        assert!(matches!(snap.status, OllamaStatus::Reachable { .. }));
        assert!(snap.models.is_empty());
    }

    #[tokio::test]
    async fn probe_reachable_but_ps_missing_expires_at_drops_dot() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "version": "0.4.0"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/ps"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "models": [
                    { "name": "qwen2.5:14b", "size_vram": 11_000_000_000_u64 }
                ]
            })))
            .mount(&server)
            .await;

        let prober = OllamaProber::new(server.uri());

        let snap = prober.probe().await;

        assert_eq!(snap.models.len(), 1);
        assert!(!snap.models[0].dot_known);
    }
}
