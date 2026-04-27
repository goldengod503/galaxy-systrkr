#[derive(Debug, Clone, Default)]
pub struct OllamaSnapshot {
    pub status: OllamaStatus,
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum OllamaStatus {
    /// No probe has completed yet.
    #[default]
    Unknown,
    Reachable {
        version: String,
    },
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub size_vram_bytes: u64,
    /// Heuristic: see spec §"Active vs idle determination."
    pub is_active: bool,
    /// false → renderer omits the activity indicator for this row.
    pub dot_known: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_snapshot_is_unknown() {
        let snap = OllamaSnapshot::default();
        assert_eq!(snap.status, OllamaStatus::Unknown);
        assert!(snap.models.is_empty());
    }
}
