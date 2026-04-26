use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Default)]
pub struct ProcSample {
    pub pid: u32,
    pub name: String,
    /// Process CPU usage in percent. sysinfo returns this as a percentage of
    /// a single core; values can exceed 100 on multithreaded workloads.
    pub cpu_pct: f32,
}

pub struct ProcSampler {
    system: System,
}

impl ProcSampler {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }

    /// Refresh process list and return top `n` by CPU usage, descending.
    pub fn top_n(&mut self, n: usize) -> Vec<ProcSample> {
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_cpu(),
        );

        let mut entries: Vec<ProcSample> = self
            .system
            .processes()
            .iter()
            .map(|(pid, p)| ProcSample {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().into_owned(),
                cpu_pct: p.cpu_usage(),
            })
            .filter(|p| p.cpu_pct > 0.05)
            .collect();

        entries.sort_by(|a, b| {
            b.cpu_pct
                .partial_cmp(&a.cpu_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.truncate(n);
        entries
    }
}

impl Default for ProcSampler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_n_returns_at_most_n_entries() {
        let mut s = ProcSampler::new();

        // Two refreshes are required for sysinfo to compute cpu_usage deltas.
        let _ = s.top_n(5);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let top = s.top_n(5);

        assert!(top.len() <= 5);
    }
}
