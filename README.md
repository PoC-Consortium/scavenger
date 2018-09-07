 <img align="right" src="https://i.imgur.com/LG63EqK.png" height="200">
 
 [![Build Status](https://travis-ci.org/PoC-Consortium/scavenger.svg?branch=master)](https://travis-ci.org/PoC-Consortium/scavenger)

# Scavenger - Burstminer in Rust

### Features
- direct io
- avx512f, avx2, avx, sse
- opencl
- fastest burstminer there is

### Requirements
- new version of rust

### Compile, test, ...

Binaries are in **target/debug** or **target/release** depending on optimazation.

``` shell
# build debug und run directly
cargo run [--features opencl]

# build debug (unoptimized)
cargo build [--features opencl]

# build release (optimized)
cargo build --release [--features opencl]

# test
cargo test  [--features opencl]
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
* JohnnyDeluxe: BURST-S338-R6VC-LTFA-2GC6G
