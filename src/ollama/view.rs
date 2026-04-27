use cosmic::iced::Length;
use cosmic::widget::{column as col, row, text};
use cosmic::Element;

use super::state::{OllamaModel, OllamaSnapshot, OllamaStatus};

pub fn section<'a, M: 'static>(snapshot: &OllamaSnapshot) -> Element<'a, M> {
    match &snapshot.status {
        OllamaStatus::Unknown => header_only("Ollama"),
        OllamaStatus::Unreachable => header_only("Ollama — not running"),
        OllamaStatus::Reachable { version } => {
            let header = text(format!("Ollama {version}")).size(13);

            if snapshot.models.is_empty() {
                col::with_children(vec![
                    header.into(),
                    text("No models loaded").size(11).into(),
                ])
                .spacing(2)
                .into()
            } else {
                let any_dot = snapshot.models.iter().any(|m| m.dot_known);
                let mut children: Vec<Element<'a, M>> = vec![header.into()];
                for m in &snapshot.models {
                    children.push(model_row(m, any_dot));
                }
                col::with_children(children).spacing(2).into()
            }
        }
    }
}

fn header_only<'a, M: 'static>(title: &str) -> Element<'a, M> {
    text(title.to_string()).size(13).into()
}

fn model_row<'a, M: 'static>(m: &OllamaModel, show_dot_column: bool) -> Element<'a, M> {
    let dot = if show_dot_column {
        let glyph = if m.dot_known {
            if m.is_active {
                "●"
            } else {
                "◯"
            }
        } else {
            " "
        };
        text(glyph.to_string()).size(11).width(Length::Fixed(14.0))
    } else {
        text(String::new()).size(11).width(Length::Fixed(0.0))
    };

    row::with_children(vec![
        dot.into(),
        text(m.name.clone()).size(11).width(Length::Fill).into(),
        text(fmt_vram(m.size_vram_bytes)).size(11).into(),
    ])
    .spacing(6)
    .into()
}

fn fmt_vram(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let f = bytes as f64;
    if f >= GIB {
        format!("{:.1} GiB", f / GIB)
    } else {
        format!("{:.0} MiB", f / MIB)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ollama::state::{OllamaModel, OllamaSnapshot, OllamaStatus};

    fn make_snap(status: OllamaStatus, models: Vec<OllamaModel>) -> OllamaSnapshot {
        OllamaSnapshot { status, models }
    }

    #[test]
    fn renders_unknown() {
        let snap = make_snap(OllamaStatus::Unknown, vec![]);
        let _: Element<'_, ()> = section(&snap);
    }

    #[test]
    fn renders_unreachable() {
        let snap = make_snap(OllamaStatus::Unreachable, vec![]);
        let _: Element<'_, ()> = section(&snap);
    }

    #[test]
    fn renders_reachable_no_models() {
        let snap = make_snap(
            OllamaStatus::Reachable { version: "0.5.7".into() },
            vec![],
        );
        let _: Element<'_, ()> = section(&snap);
    }

    #[test]
    fn renders_reachable_with_models() {
        let snap = make_snap(
            OllamaStatus::Reachable { version: "0.5.7".into() },
            vec![
                OllamaModel {
                    name: "llama3.1:8b".into(),
                    size_vram_bytes: 4_500_000_000,
                    is_active: false,
                    dot_known: true,
                },
                OllamaModel {
                    name: "qwen2.5:14b".into(),
                    size_vram_bytes: 11_000_000_000,
                    is_active: true,
                    dot_known: true,
                },
            ],
        );
        let _: Element<'_, ()> = section(&snap);
    }
}
