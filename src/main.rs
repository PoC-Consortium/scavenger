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

mod burstmath;
mod config;
mod miner;
mod plot;
mod reader;
mod requests;
mod shabals;
mod utils;
mod worker;

use config::load_cfg;
use miner::Miner;

fn main() {
    let m = Miner::new(load_cfg());
    m.run();
}
