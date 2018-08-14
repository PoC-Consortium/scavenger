extern crate serde_yaml;
extern crate sys_info;

use std::fs;
use std::u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cfg {
    pub account_id: u64,
    pub url: String,
    pub plot_dirs: Vec<String>,

    #[serde(default = "default_secret_phrase")]
    pub secret_phrase: String,

    #[serde(default = "default_worker_thread_count")]
    pub worker_thread_count: usize,

    #[serde(default = "default_reader_thread_count")]
    pub reader_thread_count: usize,

    #[serde(default = "default_gpu_worker_thread_count")]
    pub gpu_worker_thread_count: usize,

    #[serde(default = "default_gpu_platform")]
    pub gpu_platform: usize,

    #[serde(default = "default_gpu_device")]
    pub gpu_device: usize,

    #[serde(default = "default_nonces_per_cache")]
    pub nonces_per_cache: usize,

    #[serde(default = "default_target_deadline")]
    pub target_deadline: u64,

    #[serde(default = "default_use_direct_io")]
    pub use_direct_io: bool,

    #[serde(default = "default_get_mining_info_interval")]
    pub get_mining_info_interval: u64,

    #[serde(default = "default_timeout")]
    pub timeout: u64,

    #[serde(default = "default_logfile_max_count")]
    pub logfile_max_count: u32,

    #[serde(default = "default_logfile_max_size")]
    pub logfile_max_size: u64,

    /// Console log pattern
    #[serde(default = "default_console_log_pattern")]
    pub console_log_pattern: String,

    /// Logfile log pattern
    #[serde(default = "default_logfile_log_pattern")]
    pub logfile_log_pattern: String,

    #[serde(default = "default_wakeup_after")]
    pub wakeup_after: i64,

    #[serde(default = "default_console_log_level")]
    pub console_log_level: String,

    #[serde(default = "default_logfile_log_level")]
    pub logfile_log_level: String,

    #[serde(default = "default_cpu_thread_pinning")]
    pub cpu_thread_pinning: bool,
}

fn default_secret_phrase() -> String {
    "".to_owned()
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
    "{d(%H:%M:%S.%3f%z)} [{h({l}):<5}] [{T}] [{t}] - {M}:{m}{n}".to_owned()
}

fn default_logfile_log_pattern() -> String {
    "{d(%Y-%m-%dT%H:%M:%S.%3f%z)} [{h({l}):<5}] [{T}] [{f}:{L}] [{t}] - {M}:{m}{n}".to_owned()
}

fn default_worker_thread_count() -> usize {
    0
}

fn default_reader_thread_count() -> usize {
    0
}

fn default_gpu_worker_thread_count() -> usize {
    0
}

fn default_gpu_platform() -> usize {
    0
}

fn default_gpu_device() -> usize {
    0
}

fn default_nonces_per_cache() -> usize {
    65536
}

fn default_target_deadline() -> u64 {
    u64::MAX
}

fn default_use_direct_io() -> bool {
    true
}

fn default_get_mining_info_interval() -> u64 {
    3000
}

fn default_timeout() -> u64 {
    5000
}

fn default_wakeup_after() -> i64 {
    240
}

fn default_cpu_thread_pinning() -> bool {
    false
}

pub fn load_cfg(config: &str) -> Cfg {
    let cfg_str = fs::read_to_string(config).expect("failed to open config");
    let cfg: Cfg = serde_yaml::from_str(&cfg_str).expect("failed to parse config");
    if cfg.use_direct_io {
        assert!(
            cfg.nonces_per_cache % 8 == 0,
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
