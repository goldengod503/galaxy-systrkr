use cosmic::iced::Length;
use cosmic::widget::{column as col, row, text};
use cosmic::Element;

use crate::sampler::gpu::procs::GpuProcSample;
use crate::sampler::procs::ProcSample;

pub fn cpu_list<'a, M: 'static>(procs: &[ProcSample]) -> Element<'a, M> {
    let mut rows: Vec<Element<'a, M>> = vec![text("Top CPU").size(12).into()];
    if procs.is_empty() {
        rows.push(text("(idle)").size(11).into());
    } else {
        for p in procs {
            rows.push(
                row::with_children(vec![
                    text(p.name.clone()).size(11).width(Length::Fill).into(),
                    text(format!("{:.0}%", p.cpu_pct)).size(11).into(),
                ])
                .into(),
            );
        }
    }
    col::with_children(rows).spacing(2).into()
}

pub fn gpu_list<'a, M: 'static>(
    procs: &[GpuProcSample],
    backend_available: bool,
) -> Element<'a, M> {
    let mut rows: Vec<Element<'a, M>> = vec![text("Top GPU").size(12).into()];
    if !backend_available {
        rows.push(
            text("Per-process GPU not supported on this system")
                .size(11)
                .into(),
        );
    } else if procs.is_empty() {
        rows.push(text("(idle)").size(11).into());
    } else {
        for p in procs {
            let metric = match (p.utilization_pct, p.memory_bytes) {
                (Some(u), _) => format!("{u:.0}%"),
                (None, Some(b)) => fmt_bytes(b),
                _ => "—".into(),
            };
            rows.push(
                row::with_children(vec![
                    text(p.name.clone()).size(11).width(Length::Fill).into(),
                    text(metric).size(11).into(),
                ])
                .into(),
            );
        }
    }
    col::with_children(rows).spacing(2).into()
}

fn fmt_bytes(b: u64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    let f = b as f64;
    if f >= GB {
        format!("{:.1} GiB", f / GB)
    } else {
        format!("{:.0} MiB", f / MB)
    }
}
