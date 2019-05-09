#![warn(unused_extern_crates)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

mod com;
mod config;
mod cpu_worker;
mod future;
mod logger;
mod miner;
mod plot;
mod poc_hashing;
mod reader;
mod requests;
mod shabal256;
mod utils;

#[cfg(feature = "opencl")]
mod gpu_worker;
#[cfg(feature = "opencl")]
mod gpu_worker_async;
#[cfg(feature = "opencl")]
mod ocl;

use crate::config::load_cfg;
use crate::miner::Miner;
use clap::{App, Arg};
use futures::Future;
use std::process;
use tokio::runtime::Builder;

cfg_if! {
    if #[cfg(feature = "simd")] {
        extern "C" {
            pub fn init_shabal_avx512f() -> ();
            pub fn init_shabal_avx2() -> ();
            pub fn init_shabal_avx() -> ();
            pub fn init_shabal_sse2() -> ();
        }

        fn init_cpu_extensions() {
            if is_x86_feature_detected!("avx512f") {
                info!("SIMD extensions: AVX512F");
                unsafe {
                    init_shabal_avx512f();
                }
            } else if is_x86_feature_detected!("avx2") {
                info!("SIMD extensions: AVX2");
                unsafe {
                    init_shabal_avx2();
                }
            } else if is_x86_feature_detected!("avx") {
                info!("SIMD extensions: AVX");
                unsafe {
                    init_shabal_avx();
                }
            } else if is_x86_feature_detected!("sse2") {
                info!("SIMD extensions: SSE2");
                unsafe {
                    init_shabal_sse2();
                }
            } else {
                info!("SIMD extensions: none");
            }
        }
    }
}

cfg_if! {
    if #[cfg(feature = "neon")] {
        extern "C" {
            pub fn init_shabal_neon() -> ();
        }

        fn init_cpu_extensions() {
            #[cfg(target_arch = "arm")]
            let neon = is_arm_feature_detected!("neon");
            #[cfg(target_arch = "aarch64")]
            let neon = true;

            if neon {
                info!("SIMD extensions: NEON");
                unsafe {
                    init_shabal_neon();
                }
            } else {
                info!("SIMD extensions: none");
            }
        }
    }
}

fn main() {
    let arg = App::new("Scavenger - a PoC miner")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Location of the config file")
                .takes_value(true)
                .default_value("config.yaml"),
        );
    #[cfg(feature = "opencl")]
    let arg = arg.arg(
        Arg::with_name("opencl")
            .short("ocl")
            .long("opencl")
            .help("Display OpenCL platforms and devices")
            .takes_value(false),
    );

    let matches = &arg.get_matches();
    let config = matches.value_of("config").unwrap();

    let cfg_loaded = load_cfg(config);
    logger::init_logger(&cfg_loaded);

    info!("Scavenger v.{}", crate_version!());
    #[cfg(feature = "opencl")]
    info!("GPU extensions: OpenCL");

    if matches.is_present("opencl") {
        #[cfg(feature = "opencl")]
        ocl::platform_info();
        process::exit(0);
    }

    #[cfg(any(feature = "simd", feature = "neon"))]
    init_cpu_extensions();
    #[cfg(feature = "opencl")]
    ocl::gpu_info(&cfg_loaded);

    let rt = Builder::new().core_threads(1).build().unwrap();
    let m = Miner::new(cfg_loaded, rt.executor());
    m.run();
    rt.shutdown_on_idle().wait().unwrap();
}
