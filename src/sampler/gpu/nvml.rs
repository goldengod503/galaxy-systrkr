#![cfg(feature = "nvidia")]

use nvml_wrapper::Nvml as NvmlLib;
use tracing::warn;

use super::{GpuBackend, GpuSample};

pub struct Nvml {
    lib: NvmlLib,
    name: String,
    pdev: String,
    /// `true` once we've already logged a sample failure for this backend.
    sample_warned: bool,
}

impl Nvml {
    pub fn probe() -> Option<Self> {
        let lib = match NvmlLib::init() {
            Ok(l) => l,
            Err(e) => {
                warn!(error = %e, "NVML init failed; skipping NVIDIA backend");
                return None;
            }
        };
        let name = lib
            .device_by_index(0)
            .ok()
            .and_then(|d| d.name().ok())
            .unwrap_or_else(|| "NVIDIA GPU".to_string());
        let pdev = lib
            .device_by_index(0)
            .ok()
            .and_then(|d| d.pci_info().ok())
            .map(|p| p.bus_id.to_lowercase())
            .unwrap_or_default();
        Some(Self {
            lib,
            name,
            pdev,
            sample_warned: false,
        })
    }
}

impl GpuBackend for Nvml {
    fn name(&self) -> &str {
        &self.name
    }

    fn pdev(&self) -> &str {
        &self.pdev
    }

    fn is_nvidia(&self) -> bool {
        true
    }

    fn sample(&mut self) -> GpuSample {
        let device = match self.lib.device_by_index(0) {
            Ok(d) => d,
            Err(e) => {
                if !self.sample_warned {
                    warn!(error = %e, "NVML device_by_index(0) failed");
                    self.sample_warned = true;
                }
                return GpuSample::default();
            }
        };

        let utilization_pct = device.utilization_rates().ok().map(|u| u.gpu as f32);
        let mem = device.memory_info().ok();
        let temperature_c = device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .ok()
            .map(|t| t as f32);

        GpuSample {
            utilization_pct,
            memory_used_bytes: mem.as_ref().map(|m| m.used),
            memory_total_bytes: mem.as_ref().map(|m| m.total),
            temperature_c,
        }
    }
}
