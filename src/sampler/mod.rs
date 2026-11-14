//! Sampling layer: combines CPU and GPU readings into a single Sample.

pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod net;
pub mod procs;

#[derive(Clone, Debug, Default)]
pub struct Sample {
    pub cpu: CpuSample,
    pub gpu: GpuSample,
    pub net: net::NetSample,
    pub disk: disk::DiskSample,
    pub top_cpu_procs: Vec<crate::sampler::procs::ProcSample>,
    pub top_gpu_procs: Vec<crate::sampler::gpu::procs::GpuProcSample>,
}

#[derive(Clone, Debug, Default)]
pub struct CpuSample {
    pub utilization_pct: Option<f32>,
    pub temperature_c: Option<f32>,
    pub model: Option<String>,
    pub ram_used_bytes: Option<u64>,
    pub ram_total_bytes: Option<u64>,
    pub swap_used_bytes: Option<u64>,
    pub swap_total_bytes: Option<u64>,
    pub load_avg_1m: Option<f64>,
    pub load_avg_5m: Option<f64>,
    pub load_avg_15m: Option<f64>,
}

#[derive(Clone, Debug, Default)]
pub struct GpuSample {
    pub utilization_pct: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temperature_c: Option<f32>,
}

const TOP_N: usize = 5;

use cpu::CpuSampler;
use gpu::GpuBackend;

pub struct Sampler {
    cpu: CpuSampler,
    gpu_backend: Box<dyn GpuBackend>,
    proc_sampler: procs::ProcSampler,
    gpu_proc_backend: Option<Box<dyn gpu::procs::GpuProcessBackend>>,
    net: net::NetSampler,
    disk: disk::DiskSampler,
}

impl Sampler {
    pub fn new(cfg: &crate::config::SystrkrConfig) -> Self {
        let gpu_backend = gpu::probe_index(cfg.gpu_index);
        let gpu_proc_backend = gpu::procs::probe(
            gpu_backend.pdev(),
            gpu_backend.is_nvidia(),
            gpu_backend.nvml_index(),
        );
        Self {
            cpu: CpuSampler::new(),
            gpu_backend,
            proc_sampler: procs::ProcSampler::new(),
            gpu_proc_backend,
            net: net::NetSampler::new(),
            disk: disk::DiskSampler::new(),
        }
    }

    pub fn gpu_name(&self) -> &str {
        self.gpu_backend.name()
    }

    pub fn gpu_proc_backend_available(&self) -> bool {
        self.gpu_proc_backend.is_some()
    }

    /// Per-process scans walk all of `/proc` and dominate tick cost, so they
    /// only run when the caller has somewhere to show them (`with_procs`).
    pub fn tick(&mut self, with_procs: bool) -> Sample {
        let (top_cpu_procs, top_gpu_procs) = if with_procs {
            self.top_procs()
        } else {
            Default::default()
        };
        Sample {
            cpu: self.cpu.tick(),
            gpu: self.gpu_backend.sample(),
            net: self.net.tick(),
            disk: self.disk.tick(),
            top_cpu_procs,
            top_gpu_procs,
        }
    }

    /// Usage figures are deltas against the previous scan, so the first call
    /// after a gap reports an average over that gap (or nothing, for
    /// processes not seen before) and the next one is accurate.
    pub fn top_procs(&mut self) -> (Vec<procs::ProcSample>, Vec<gpu::procs::GpuProcSample>) {
        (
            self.proc_sampler.top_n(TOP_N),
            self.gpu_proc_backend
                .as_mut()
                .map(|b| b.top_n(TOP_N))
                .unwrap_or_default(),
        )
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new(&crate::config::SystrkrConfig::default())
    }
}

#[cfg(test)]
mod aggregator_tests {
    use super::*;

    fn burn_cpu(d: std::time::Duration) {
        let end = std::time::Instant::now() + d;
        while std::time::Instant::now() < end {
            std::hint::black_box(0u64);
        }
    }

    #[test]
    fn two_consecutive_ticks_dont_panic() {
        let mut s = Sampler::new(&crate::config::SystrkrConfig::default());

        let _ = s.tick(true);
        let _ = s.tick(true);
    }

    #[test]
    fn tick_without_procs_returns_empty_process_lists() {
        let mut s = Sampler::new(&crate::config::SystrkrConfig::default());
        let _ = s.tick(false);
        burn_cpu(std::time::Duration::from_millis(300));

        let sample = s.tick(false);

        assert!(sample.top_cpu_procs.is_empty());
        assert!(sample.top_gpu_procs.is_empty());
    }

    #[test]
    fn tick_with_procs_after_skipped_ticks_reports_busy_process() {
        let mut s = Sampler::new(&crate::config::SystrkrConfig::default());
        let _ = s.top_procs();
        let _ = s.tick(false);
        burn_cpu(std::time::Duration::from_millis(300));

        let sample = s.tick(true);

        let me = std::process::id();
        assert!(sample.top_cpu_procs.iter().any(|p| p.pid == me));
    }
}
