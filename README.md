 <img align="right" src="https://i.imgur.com/LG63EqK.png" height="200">
 
 [![Build Status](https://travis-ci.org/PoC-Consortium/scavenger.svg?branch=master)](https://travis-ci.org/PoC-Consortium/scavenger)

# Scavenger - Burstminer in Rust

### Features
- windows, linux, macOS, android & more
- x86 32 & 64bit, arm, aarch64 
- direct io
- avx512f, avx2, avx, sse
- opencl
- fastest burstminer there is

### Requirements
- new version of rust

### Compile, test, ...

Binaries are in **target/debug** or **target/release** depending on optimazation.

``` shell
# decide on features to run/build:
simd: support for SSE2, AVX, AVX2 and AVX512F (x86_cpu)
neon: support for Arm NEON (cpu)
opencl: support for OpenCL (gpu)

# build debug und run directly
e.g. cargo run --features=simd    #for a cpu version with SIMD support

# build debug (unoptimized)
e.g cargo build --features=neon   #for a arm cpu version with NEON support

# build release (optimized)
e.g. cargo build --release --features=opencl,simd    #for a cpu/gpu version

# test
cargo test  [--features={opencl,simd,neon}]
```

### Run

```shell
scavenger --help
```

### Config

The miner needs a **config.yaml** file with the following structure:

https://github.com/PoC-Consortium/scavenger/blob/master/config.yaml

### Docker

A docker image based on alpine linux is built automatically on every commit to master: `spebern/scavenger`
This image will use only your cpu.

To run it on the fly use something like this:
```
docker run \
--rm \
--name scavenger \
--volume /path/to/your/config.yaml:/data/config.yaml \
--volume /path/to/your/disks:/disks \
spebern/scavenger
```

Alternatively a docker compose file could look like this:
```
version: '2'
services:
  scavenger:
    image: spebern/scavenger
    restart: always
    volumes:
      - /path/to/your/disks:/disks
      - /path/to/your/config.yaml:/data/config.yaml
```

### Donate 
* bold: BURST-8V9Y-58B4-RVWP-8HQAV
  - architecture
  - linux support
* JohnnyDeluxe: BURST-S338-R6VC-LTFA-2GC6G
  - open cl
  - direct io
  - shabal optimizations
  - windows support
