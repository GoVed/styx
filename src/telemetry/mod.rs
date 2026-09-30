use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use sysinfo::{Disks, System};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTelemetry {
    pub name: String,
    #[serde(default = "default_vendor")]
    pub vendor: String,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub vram_pct: f32,
    pub gpu_util_pct: f32,
    pub temperature_c: Option<u32>,
}

fn default_vendor() -> String {
    "auto".to_string()
}

fn default_concurrency() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetry {
    pub host_cpu_pct: f32,
    pub host_cpu_cores: usize,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_pct: f32,
    pub disk_used_gb: u64,
    pub disk_total_gb: u64,
    pub disk_pct: f32,
    pub uptime_secs: u64,
    pub running_containers: usize,
    pub active_mcp_servers: usize,
    pub pending_approvals: usize,
    pub active_model: String,
    pub tokens_per_second: f32,
    pub gpu: Option<GpuTelemetry>,
    #[serde(default)]
    pub queued_turns: usize,
    #[serde(default)]
    pub active_turns: usize,
    #[serde(default = "default_concurrency")]
    pub max_concurrent_turns: usize,
}

#[derive(Clone)]
pub struct TelemetryCollector {
    system: Arc<RwLock<System>>,
    disks: Arc<RwLock<Disks>>,
    token_counter: Arc<AtomicU64>,
    last_tokens_per_sec: Arc<RwLock<f32>>,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();

        Self {
            system: Arc::new(RwLock::new(sys)),
            disks: Arc::new(RwLock::new(disks)),
            token_counter: Arc::new(AtomicU64::new(0)),
            last_tokens_per_sec: Arc::new(RwLock::new(0.0)),
        }
    }

    pub fn record_tokens(&self, count: u64) {
        self.token_counter.fetch_add(count, Ordering::Relaxed);
    }

    pub async fn update_tok_rate(&self, rate: f32) {
        let mut w = self.last_tokens_per_sec.write().await;
        *w = rate;
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn collect_snapshot(
        &self,
        running_containers: usize,
        active_mcp_servers: usize,
        pending_approvals: usize,
        active_model: String,
        queued_turns: usize,
        active_turns: usize,
        max_concurrent_turns: usize,
    ) -> SystemTelemetry {
        let mut sys = self.system.write().await;
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let cpus = sys.cpus();
        let cpu_cores = cpus.len();
        let host_cpu_pct = if cpu_cores > 0 {
            let sum: f32 = cpus.iter().map(|c| c.cpu_usage()).sum();
            (sum / cpu_cores as f32).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let memory_total_mb = sys.total_memory() / (1024 * 1024);
        let memory_used_mb = sys.used_memory() / (1024 * 1024);
        let memory_pct = if memory_total_mb > 0 {
            (memory_used_mb as f32 / memory_total_mb as f32) * 100.0
        } else {
            0.0
        };

        let mut disks = self.disks.write().await;
        disks.refresh(false);
        let mut disk_total_bytes: u64 = 0;
        let mut disk_avail_bytes: u64 = 0;
        for d in disks.iter() {
            disk_total_bytes += d.total_space();
            disk_avail_bytes += d.available_space();
        }
        let disk_used_bytes = disk_total_bytes.saturating_sub(disk_avail_bytes);
        let disk_total_gb = disk_total_bytes / (1024 * 1024 * 1024);
        let disk_used_gb = disk_used_bytes / (1024 * 1024 * 1024);
        let disk_pct = if disk_total_gb > 0 {
            (disk_used_gb as f32 / disk_total_gb as f32) * 100.0
        } else {
            0.0
        };

        let uptime_secs = System::uptime();
        let tokens_per_second = *self.last_tokens_per_sec.read().await;

        let gpu = Self::probe_gpu().await;

        SystemTelemetry {
            host_cpu_pct: (host_cpu_pct * 10.0).round() / 10.0,
            host_cpu_cores: cpu_cores,
            memory_used_mb,
            memory_total_mb,
            memory_pct: (memory_pct * 10.0).round() / 10.0,
            disk_used_gb,
            disk_total_gb,
            disk_pct: (disk_pct * 10.0).round() / 10.0,
            uptime_secs,
            running_containers,
            active_mcp_servers,
            pending_approvals,
            active_model,
            tokens_per_second,
            gpu,
            queued_turns,
            active_turns,
            max_concurrent_turns,
        }
    }

    async fn probe_gpu() -> Option<GpuTelemetry> {
        // Probe NVIDIA first; if not present, probe AMD ROCm
        if let Some(gpu) = Self::probe_nvidia_gpu().await {
            return Some(gpu);
        }
        Self::probe_amd_gpu().await
    }

    async fn probe_nvidia_gpu() -> Option<GpuTelemetry> {
        // Query nvidia-smi if available
        let output = tokio::process::Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,memory.used,memory.total,utilization.gpu,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .await
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let line = text.lines().next()?;
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 5 {
            return None;
        }

        let name = parts[0].to_string();
        let vram_used_mb: u64 = parts[1].parse().unwrap_or(0);
        let vram_total_mb: u64 = parts[2].parse().unwrap_or(1);
        let gpu_util_pct: f32 = parts[3].parse().unwrap_or(0.0);
        let temp: Option<u32> = parts[4].parse().ok();

        let vram_pct = if vram_total_mb > 0 {
            (vram_used_mb as f32 / vram_total_mb as f32) * 100.0
        } else {
            0.0
        };

        Some(GpuTelemetry {
            name,
            vendor: "nvidia".to_string(),
            vram_used_mb,
            vram_total_mb,
            vram_pct: (vram_pct * 10.0).round() / 10.0,
            gpu_util_pct,
            temperature_c: temp,
        })
    }

    async fn probe_amd_gpu() -> Option<GpuTelemetry> {
        let drm_dir = std::path::Path::new("/sys/class/drm");
        if !drm_dir.exists() {
            return None;
        }

        let mut best_card: Option<std::path::PathBuf> = None;
        let mut max_vram: u64 = 0;

        if let Ok(entries) = std::fs::read_dir(drm_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if file_name.starts_with("card") && !file_name.contains('-') {
                    let vram_path = path.join("device/mem_info_vram_total");
                    if let Ok(vram_str) = std::fs::read_to_string(&vram_path)
                        && let Ok(vram) = vram_str.trim().parse::<u64>()
                            && vram > max_vram {
                                max_vram = vram;
                                best_card = Some(path.clone());
                            }
                }
            }
        }

        let card_path = best_card?;
        let vram_total_mb = max_vram / (1024 * 1024);
        if vram_total_mb == 0 {
            return None;
        }

        let vram_used_mb = std::fs::read_to_string(card_path.join("device/mem_info_vram_used"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0)
            / (1024 * 1024);

        let gpu_util_pct = std::fs::read_to_string(card_path.join("device/gpu_busy_percent"))
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(0.0);

        // Read temperature if available from hwmon
        let mut temp_c: Option<u32> = None;
        if let Ok(hwmon_entries) = std::fs::read_dir(card_path.join("device/hwmon")) {
            for entry in hwmon_entries.flatten() {
                let temp_file = entry.path().join("temp1_input");
                if let Ok(t_str) = std::fs::read_to_string(temp_file)
                    && let Ok(t_millicelsius) = t_str.trim().parse::<u32>() {
                        temp_c = Some(t_millicelsius / 1000);
                        break;
                    }
            }
        }

        // Try to get friendly GPU product name via rocm-smi
        let mut name = "AMD Radeon GPU (ROCm)".to_string();
        if let Ok(out) = tokio::process::Command::new("rocm-smi")
            .args(["--showproductname", "--json"])
            .output()
            .await
            && out.status.success() {
                let txt = String::from_utf8_lossy(&out.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&txt)
                    && let Some(obj) = json.as_object() {
                        for (_k, v) in obj {
                            if let Some(series) = v.get("Card Series").and_then(|s| s.as_str())
                                && !series.contains("Processor") {
                                    name = format!("{} (ROCm)", series);
                                    break;
                                }
                        }
                    }
            }

        let vram_pct = if vram_total_mb > 0 {
            (vram_used_mb as f32 / vram_total_mb as f32) * 100.0
        } else {
            0.0
        };

        Some(GpuTelemetry {
            name,
            vendor: "amd".to_string(),
            vram_used_mb,
            vram_total_mb,
            vram_pct: (vram_pct * 10.0).round() / 10.0,
            gpu_util_pct,
            temperature_c: temp_c,
        })
    }
}
