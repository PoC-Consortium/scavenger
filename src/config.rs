extern crate num_cpus;
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

    #[serde(default = "default_nonces_per_cache")]
    pub nonces_per_cache: usize,

    #[serde(default = "default_target_deadline")]
    pub target_deadline: u64,

    #[serde(default = "default_use_direct_io")]
    pub use_direct_io: bool,

    #[serde(default = "default_get_mining_info_interval")]
    pub get_mining_info_interval: u64,
}

fn default_secret_phrase() -> String {
    "".to_owned()
}

fn default_worker_thread_count() -> usize {
    num_cpus::get() + 1
}

fn default_reader_thread_count() -> usize {
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

pub fn load_cfg() -> Cfg {
    let cfg_str = fs::read_to_string("config.yaml").expect("failed to find config.yaml");
    let cfg: Cfg = serde_yaml::from_str(&cfg_str).expect("failed to parse config yaml");
    if cfg.use_direct_io {
        assert!(
            cfg.nonces_per_cache % 8 == 0,
            "nonces_per_cache must be devisable by 8 when using direct io"
        );
    }
    cfg
}
