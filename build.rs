extern crate cc;

fn main() {
    let mut shared_config = cc::Build::new();

    #[cfg(target_env = "msvc")]
    shared_config
        .flag("/O2")
        .flag("/Oi")
        .flag("/Ot")
        .flag("/Oy")
        .flag("/GT")
        .flag("/GL");

    #[cfg(not(target_env = "msvc"))]
    shared_config.flag("-std=c99").flag("-mtune=native");

    let mut config = shared_config.clone();

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/mshabal_128_sse2.c")
        .file("src/c/shabal_sse2.c")
        .compile("shabal_sse");

    let mut config = shared_config.clone();

    #[cfg(target_env = "msvc")]
    config.flag("/arch:AVX");

    #[cfg(not(target_env = "msvc"))]
    config.flag("-mavx");

    config
        .file("src/c/mshabal_128_avx.c")
        .file("src/c/shabal_avx.c")
        .compile("shabal_avx");

    let mut config = shared_config.clone();

    #[cfg(target_env = "msvc")]
    config.flag("/arch:AVX2");

    #[cfg(not(target_env = "msvc"))]
    config.flag("-mavx2");

    config
        .file("src/c/mshabal_256_avx2.c")
        .file("src/c/shabal_avx2.c")
        .compile("shabal_avx2");

    let mut config = shared_config.clone();

    #[cfg(target_env = "msvc")]
    config.flag("/arch:AVX512F");

    #[cfg(not(target_env = "msvc"))]
    config.flag("-mavx512f");

    config
        .file("src/c/mshabal_512_avx512f.c")
        .file("src/c/shabal_avx512f.c")
        .compile("shabal_avx512f");

    let mut config = shared_config.clone();

    config.file("src/c/shabal.c").compile("shabal_1d");

    #[cfg(feature = "neon")]
    let mut config = shared_config.clone();

    #[cfg(all(not(target_env = "msvc"), feature = "neon"))]
    config.flag("-mfpu=neon");

    #[cfg(feature = "neon")]
    config
        .file("src/c/mshabal_128_neon.c")
        .file("src/c/shabal_neon.c")
        .compile("shabal_neon");
}
