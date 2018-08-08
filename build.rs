extern crate cc;

fn main() {
    let mut config = cc::Build::new();

    #[cfg(target_env = "msvc")]
    config.flag("/arch:AVX2");
    #[cfg(not(target_env = "msvc"))]
    config.flag("-std=c99").flag("-march=native");

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/mshabal_128.c");

    if is_x86_feature_detected!("avx") {
        config.file("src/c/mshabal_256.c").file("src/c/shabal.c");
    } else {
        config.file("src/c/shabal_sse2.c");
    }

    config.compile("libshabal.a");
}
