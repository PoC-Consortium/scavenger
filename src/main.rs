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
extern crate log4rs;

mod burstmath;
mod config;
mod logger;
mod miner;
mod plot;
mod reader;
mod requests;
mod shabals;
mod utils;
mod worker;

use clap::{App, Arg};
use config::load_cfg;
use miner::Miner;

fn main() {
    logger::init_logger();
    info!("Scavenger v.{}", "1.0");
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
    let m = Miner::new(load_cfg(config));
    m.run();
}
