//! Device and machine-tier selection for local diffusion.
//!
//! The defaults are intentionally conservative: SDXL Turbo in fp16 needs roughly
//! 8-10 GB of working memory, so a GPU is only used when the machine has at least
//! [`MIN_GPU_MEMORY_GB`]. Everything else falls back to the CPU, and only when the
//! caller explicitly allows it.

use anyhow::{Result, bail};
use candle_core::{DType, Device};
use std::fmt::{Display, Formatter};
use sysinfo::System;

/// Minimum total system memory (GiB) before the GPU path is attempted.
pub const MIN_GPU_MEMORY_GB: f64 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    // These variants only exist in builds with the matching feature; allow them to
    // be "unconstructed" in other builds instead of cfg-ing the whole enum.
    #[cfg_attr(not(feature = "metal"), allow(dead_code))]
    Metal,
    #[cfg_attr(not(feature = "cuda"), allow(dead_code))]
    Cuda,
    Cpu,
}

impl Display for DeviceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceKind::Metal => write!(f, "Metal"),
            DeviceKind::Cuda => write!(f, "CUDA"),
            DeviceKind::Cpu => write!(f, "CPU"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    /// A usable GPU and enough memory: SDXL Turbo in fp16 at the requested size.
    GpuTurbo,
    /// No usable GPU, or too little memory: CPU only with `--allow-cpu`.
    CpuFallback,
}

/// Pure tier decision, kept separate from device probing so it can be tested with
/// injected memory values.
pub fn pick_tier(kind: DeviceKind, total_memory_gb: f64) -> Tier {
    if kind != DeviceKind::Cpu && total_memory_gb >= MIN_GPU_MEMORY_GB {
        Tier::GpuTurbo
    } else {
        Tier::CpuFallback
    }
}

/// Total system memory in GiB.
///
/// On unified-memory systems (Apple Silicon) this is the right signal. On
/// discrete-GPU systems it is a proxy for VRAM, as agreed for this task.
pub fn total_memory_gb() -> f64 {
    let mut system = System::new();
    system.refresh_memory();
    system.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0)
}

/// A device that has been probed but not yet tier-checked.
#[derive(Clone)]
pub struct SelectedDevice {
    pub device: Device,
    pub kind: DeviceKind,
    pub dtype: DType,
    pub total_memory_gb: f64,
}

impl SelectedDevice {
    pub fn description(&self) -> String {
        format!(
            "{} ({:?}, {:.1} GiB system memory)",
            self.kind, self.dtype, self.total_memory_gb
        )
    }
}

fn gpu_device() -> Option<(Device, DeviceKind)> {
    #[cfg(feature = "metal")]
    if let Ok(device) = Device::new_metal(0) {
        return Some((device, DeviceKind::Metal));
    }

    #[cfg(feature = "cuda")]
    if let Ok(device) = Device::new_cuda(0) {
        return Some((device, DeviceKind::Cuda));
    }

    None
}

/// Probe the best compiled-in device without applying the memory tier.
///
/// Used by `--dry-run`, which must succeed on a GPU-less CI machine.
pub fn detect_device() -> SelectedDevice {
    let total_memory_gb = total_memory_gb();
    match gpu_device() {
        Some((device, kind)) => SelectedDevice {
            device,
            kind,
            dtype: DType::F16,
            total_memory_gb,
        },
        None => SelectedDevice {
            device: Device::Cpu,
            kind: DeviceKind::Cpu,
            dtype: DType::F32,
            total_memory_gb,
        },
    }
}

/// Resolve the device used for a real generation run.
///
/// A GPU is used when the machine has at least [`MIN_GPU_MEMORY_GB`]. Otherwise
/// this returns a friendly error unless `allow_cpu` is set, in which case the run
/// continues on the CPU in F32 (fp16 weights are converted on load).
pub fn require_generation_device(allow_cpu: bool) -> Result<SelectedDevice> {
    let detected = detect_device();
    match pick_tier(detected.kind, detected.total_memory_gb) {
        Tier::GpuTurbo => Ok(detected),
        Tier::CpuFallback => {
            let reason = if detected.kind == DeviceKind::Cpu {
                "this build has no GPU backend compiled in. macOS builds include Metal \
                 automatically; on Linux rebuild with --features cuda,cudnn"
                    .to_string()
            } else {
                format!(
                    "{} was found but the machine has only {:.1} GiB of memory (need {:.0} GiB)",
                    detected.kind, detected.total_memory_gb, MIN_GPU_MEMORY_GB
                )
            };
            if !allow_cpu {
                bail!(
                    "SDXL Turbo cannot run safely: {reason}. \
                     Pass --allow-cpu to run on the CPU (slow and memory hungry), \
                     or use a smaller --width/--height."
                );
            }
            Ok(SelectedDevice {
                device: Device::Cpu,
                kind: DeviceKind::Cpu,
                dtype: DType::F32,
                total_memory_gb: detected.total_memory_gb,
            })
        }
    }
}

/// Whether auto mode may select the SDXL generator on this machine.
pub fn can_run_sdxl() -> bool {
    let detected = detect_device();
    pick_tier(detected.kind, detected.total_memory_gb) == Tier::GpuTurbo
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_tier_gpu_needs_enough_memory() {
        assert_eq!(pick_tier(DeviceKind::Metal, 16.0), Tier::GpuTurbo);
        assert_eq!(pick_tier(DeviceKind::Cuda, 64.0), Tier::GpuTurbo);
        assert_eq!(pick_tier(DeviceKind::Metal, 15.9), Tier::CpuFallback);
        assert_eq!(pick_tier(DeviceKind::Cuda, 8.0), Tier::CpuFallback);
    }

    #[test]
    fn test_pick_tier_cpu_always_falls_back() {
        assert_eq!(pick_tier(DeviceKind::Cpu, 128.0), Tier::CpuFallback);
    }

    #[test]
    fn test_total_memory_is_positive() {
        assert!(total_memory_gb() > 0.0);
    }

    #[test]
    fn test_require_generation_device_respects_allow_cpu() {
        let detected = detect_device();
        let result = require_generation_device(false);
        match pick_tier(detected.kind, detected.total_memory_gb) {
            Tier::GpuTurbo => assert!(result.is_ok()),
            Tier::CpuFallback => {
                assert!(result.is_err());
                let device = require_generation_device(true).unwrap();
                assert_eq!(device.kind, DeviceKind::Cpu);
                assert_eq!(device.dtype, DType::F32);
            }
        }
    }
}
