use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use super::CpuSample;

pub struct CpuSampler {
    system: System,
    components: sysinfo::Components,
    model: Option<String>,
}

impl CpuSampler {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        system.refresh_cpu_all();
        let model = system.cpus().first().map(|c| c.brand().to_string());
        let components = sysinfo::Components::new_with_refreshed_list();
        Self {
            system,
            components,
            model,
        }
    }

    pub fn tick(&mut self) -> CpuSample {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.components.refresh();

        let utilization_pct = average_cpu_pct(&self.system);
        let temperature_c = pick_cpu_temperature(&self.components);

        let load = sysinfo::System::load_average();

        CpuSample {
            utilization_pct,
            temperature_c,
            model: self.model.clone(),
            ram_used_bytes: Some(self.system.used_memory()),
            ram_total_bytes: Some(self.system.total_memory()),
            swap_used_bytes: Some(self.system.used_swap()),
            swap_total_bytes: Some(self.system.total_swap()),
            load_avg_1m: Some(load.one),
            load_avg_5m: Some(load.five),
            load_avg_15m: Some(load.fifteen),
        }
    }
}

impl Default for CpuSampler {
    fn default() -> Self {
        Self::new()
    }
}

fn average_cpu_pct(system: &System) -> Option<f32> {
    let cpus = system.cpus();
    if cpus.is_empty() {
        return None;
    }
    let sum: f32 = cpus.iter().map(|c| c.cpu_usage()).sum();
    Some(sum / cpus.len() as f32)
}

fn pick_cpu_temperature(components: &sysinfo::Components) -> Option<f32> {
    // sysinfo 0.32 returns f32 (not Option<f32>); use NaN/zero as "no value".
    let valid = |t: f32| t.is_finite() && t > 0.0;
    let preferred = ["package id 0", "tdie", "tctl", "cpu"];
    for needle in preferred {
        for c in components.iter() {
            if c.label().to_lowercase().contains(needle) {
                let t = c.temperature();
                if valid(t) {
                    return Some(t);
                }
            }
        }
    }
    components.iter().map(|c| c.temperature()).find(|t| valid(*t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_twice_does_not_panic_and_returns_a_sample() {
        let mut sampler = CpuSampler::new();

        let _first = sampler.tick();
        let second = sampler.tick();

        assert!(second.ram_total_bytes.unwrap_or(0) > 0);
    }
}
