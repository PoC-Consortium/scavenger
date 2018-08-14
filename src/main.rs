#[macro_use]
extern crate serde_derive;
extern crate chan;
extern crate futures;
extern crate hex;
extern crate hyper;
extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate stopwatch;
extern crate tokio;
extern crate tokio_core;
extern crate url;
#[macro_use]
extern crate cfg_if;
extern crate filetime;
#[macro_use]
extern crate clap;
extern crate rand;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate core_affinity;
extern crate log4rs;

mod burstmath;
mod config;
mod logger;
mod miner;
mod ocl;
mod plot;
mod reader;
mod requests;
mod shabals;
mod utils;
mod worker;

use clap::{App, Arg};
use config::load_cfg;
use miner::Miner;

extern "C" {
    pub fn init_shabal_avx2() -> ();

    pub fn init_shabal_avx() -> ();

    pub fn init_shabal_sse2() -> ();
}

fn init_simd_extensions() {
    if is_x86_feature_detected!("avx2") {
        info!("SIMD extensions: AVX2");
        unsafe {
            init_shabal_avx2();
        }
    } else if is_x86_feature_detected!("avx") {
        info!("SIMD extensions: AVX");
        unsafe {
            init_shabal_avx();
        }
    } else {
        info!("SIMD extensions: SSE2");
        unsafe {
            init_shabal_sse2();
        }
    }
}

fn main() {
    let matches = App::new("Scavenger - a Burst miner")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Location of the config file")
                .takes_value(true),
        )
        .get_matches();

    let config = matches.value_of("config").unwrap_or("config.yaml");

    info!("Scavenger v.{}", "1.0");

    let cfg_loaded = load_cfg(config);
    logger::init_logger(&cfg_loaded);

    init_simd_extensions();

    ocl::init_gpu(&cfg_loaded);

    let m = Miner::new(cfg_loaded);
    m.run();
}
