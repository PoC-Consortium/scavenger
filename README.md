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

``` yaml
# secret phrase of account, leave out if pool mining
secret_phrase: 'your burst accounts secret phrase'

# numeric account id
account_id: 10282355196851764065

# list of directories containing plot files
plot_dirs:
  - 'C:\first\windows\plot\dir'
  - 'C:\second\windows\plot\dir'
  - '/first/linux/plot/dir'
  - '/second/linux/plot/dir'

# url for getting mining info + submitting nonces
url: 'http://pool.dev.burst-test.net:8124'

# threads to use for calculating deadlines | defaults to num cores + 1
worker_thread_count: 2

# threads to use for reading from disks | defaults to number of disks plotfiles are spread over
reader_thread_count: 3

# nonces to read in at once
# there will be worker_thread_count * 2 buffers in total
# to calculate ram size used for caching: nonces_per_cache * worker_thread_count * 2 * 64
nonces_per_cache: 65536 # default 65536

# deadline limit | defaults to max u64
target_deadline: 10885484741537822773

# avoid operating system caching
# the nonces in your plotfile need to be multiple of 8
use_direct_io: true # default true

# interval for getting mining info [ms]
get_mining_info_interval: 3000 # default 3000ms

# timeout requests to pool/wallet [ms]
timeout: 5000  # default 5000ms

# delay after disks will be woken up (if 0 no wakeup) [s]
wakeup_after: 240  # default 240s

# pin worker threads to cpu cores
cpu_thread_pinning: false # pin worker threads to cpu cores | default false
```

### Donate 
* bold: BURST-8V9Y-58B4-RVWP-8HQAV
  * implementation
* JohnnyDeluxe: BURST-S338-R6VC-LTFA-2GC6G
  * shabal optimizations
  * direct io
  * windows support
  * countless ideas and optimization strategies
