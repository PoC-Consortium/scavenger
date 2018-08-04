extern crate cc;
use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS");

    let mut config = cc::Build::new();

    if target_os == Ok("macos".to_string()) {
        config.file("src/c/shabal64-darwin.s");
    } else {
        config.file("src/c/shabal64.s");
    }

    config
        .file("src/c/mshabal_128.c")
        .file("src/c/mshabal_256.c")
        .file("src/c/shabal.c")
        .flag("-mavx2")
        .flag("-std=c99")
        .compile("libshabal.a");
}
