 <img align="right" src="https://i.imgur.com/LG63EqK.png" height="200">
 
 [![Build Status](https://travis-ci.org/PoC-Consortium/scavenger.svg?branch=master)](https://travis-ci.org/PoC-Consortium/scavenger)

# Scavenger - Burstminer in Rust

### Features
- direct io
- avx2, avx, sse
- fastest burstminer there is

### Requirements
- new version of rust

### Compile, test, ...

Binaries are in **target/debug** or **target/release** depending on optimazation.

``` shell
# build debug und run directly
cargo run

# build debug (unoptimized)
cargo build

# build release (optimized)
cargo build --release

# test
cargo test
```

### Run

```shell
scavenger --help
```

### Config

The miner needs a **config.yaml** file with the following structure:

https://github.com/PoC-Consortium/scavenger/blob/master/config.yaml

### Donate 
* bold: BURST-8V9Y-58B4-RVWP-8HQAV
  * implementation
* JohnnyDeluxe: BURST-S338-R6VC-LTFA-2GC6G
  * shabal optimizations
  * direct io
  * windows support
  * countless ideas and optimization strategies
