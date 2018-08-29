extern crate serde_yaml;
extern crate sys_info;

use std::collections::HashMap;
use std::fs;
use std::u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cfg {
    #[serde(default = "default_secret_phrase")]
    pub account_id_to_secret_phrase: HashMap<u64, String>,

    pub plot_dirs: Vec<String>,
    pub url: String,

    #[serde(default = "default_hdd_reader_thread_count")]
    pub hdd_reader_thread_count: usize,

    #[serde(default = "default_hdd_use_direct_io")]
    pub hdd_use_direct_io: bool,

    #[serde(default = "default_hdd_wakeup_after")]
    pub hdd_wakeup_after: i64,

    #[serde(default = "default_cpu_worker_thread_count")]
    pub cpu_worker_thread_count: usize,

    #[serde(default = "default_cpu_nonces_per_cache")]
    pub cpu_nonces_per_cache: usize,

    #[serde(default = "default_cpu_thread_pinning")]
    pub cpu_thread_pinning: bool,

    #[serde(default = "default_gpu_platform")]
    pub gpu_platform: usize,

    #[serde(default = "default_gpu_device")]
    pub gpu_device: usize,

    #[serde(default = "default_gpu_worker_thread_count")]
    pub gpu_worker_thread_count: usize,

    #[serde(default = "default_gpu_nonces_per_cache")]
    pub gpu_nonces_per_cache: usize,

    #[serde(default = "default_gpu_mem_mapping")]
    pub gpu_mem_mapping: bool,

    #[serde(default = "default_target_deadline")]
    pub target_deadline: u64,

    #[serde(default = "default_get_mining_info_interval")]
    pub get_mining_info_interval: u64,

    #[serde(default = "default_timeout")]
    pub timeout: u64,

    #[serde(default = "default_console_log_level")]
    pub console_log_level: String,

    #[serde(default = "default_logfile_log_level")]
    pub logfile_log_level: String,

    #[serde(default = "default_logfile_max_count")]
    pub logfile_max_count: u32,

    #[serde(default = "default_logfile_max_size")]
    pub logfile_max_size: u64,

    #[serde(default = "default_console_log_pattern")]
    pub console_log_pattern: String,

    #[serde(default = "default_logfile_log_pattern")]
    pub logfile_log_pattern: String,

    #[serde(default = "default_show_progress")]
    pub show_progress: bool,

    #[serde(default = "default_show_drive_stats")]
    pub show_drive_stats: bool,

    #[serde(default = "default_benchmark_only")]
    pub benchmark_only: String,
}

fn default_secret_phrase() -> HashMap<u64, String> {
    HashMap::new()
}

fn default_hdd_reader_thread_count() -> usize {
    0
}

fn default_hdd_use_direct_io() -> bool {
    true
}

fn default_hdd_wakeup_after() -> i64 {
    240
}

fn default_cpu_worker_thread_count() -> usize {
    0
}

fn default_cpu_nonces_per_cache() -> usize {
    65536
}

fn default_cpu_thread_pinning() -> bool {
    false
}

fn default_gpu_platform() -> usize {
    0
}

fn default_gpu_device() -> usize {
    0
}

fn default_gpu_worker_thread_count() -> usize {
    0
}

fn default_gpu_nonces_per_cache() -> usize {
    1_048_576
}

fn default_gpu_mem_mapping() -> bool {
    false
}

fn default_target_deadline() -> u64 {
    u64::from(u32::MAX)
}

fn default_get_mining_info_interval() -> u64 {
    3000
}

fn default_timeout() -> u64 {
    5000
}

fn default_console_log_level() -> String {
    "Info".to_owned()
}

fn default_logfile_log_level() -> String {
    "Warn".to_owned()
}

fn default_logfile_max_count() -> u32 {
    10
}

fn default_logfile_max_size() -> u64 {
    20
}

fn default_console_log_pattern() -> String {
    "\r{d(%H:%M:%S.%3f%z)} [{h({l}):<5}] [{T}] [{t}] - {M}:{m}{n}".to_owned()
}

fn default_logfile_log_pattern() -> String {
    "\r{d(%Y-%m-%dT%H:%M:%S.%3f%z)} [{h({l}):<5}] [{T}] [{f}:{L}] [{t}] - {M}:{m}{n}".to_owned()
}

fn default_show_progress() -> bool {
    true
}

fn default_show_drive_stats() -> bool {
    false
}

fn default_benchmark_only() -> String {
    "disabled".to_owned()
}

pub fn load_cfg(config: &str) -> Cfg {
    let cfg_str = fs::read_to_string(config).expect("failed to open config");
    let cfg: Cfg = serde_yaml::from_str(&cfg_str).expect("failed to parse config");
    if cfg.hdd_use_direct_io {
        assert!(
            cfg.cpu_nonces_per_cache % 8 == 0 && cfg.gpu_nonces_per_cache % 8 == 0,
            "nonces_per_cache must be devisable by 8 when using direct io"
        );
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_cfg() {
        let cfg = load_cfg("config.yaml");
        assert_eq!(cfg.timeout, 5000);
        assert_eq!(cfg.plot_dirs, vec!["test_data"]);
    }
}
