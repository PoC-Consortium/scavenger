extern crate cc;

fn main() {
    let mut config = cc::Build::new();

    #[cfg(target_env = "msvc")]
    config
        .flag("/O2")
        .flag("/Oi")
        .flag("/Ot")
        .flag("/Oy")
        .flag("/GT")
        .flag("/GL");

    #[cfg(not(target_env = "msvc"))]
    config.flag("-std=c99").flag("-mtune=native");

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/mshabal_128.c")
        .file("src/c/shabal_sse2.c")
        .file("src/c/shabal_avx.c")
        .compile("shabal.a");

    #[cfg(target_env = "msvc")]
    config.flag("/arch:AVX2");

    #[cfg(not(target_env = "msvc"))]
    config.flag("-mavx2");

    config
        .clone()
        .file("src/c/mshabal_256.c")
        .file("src/c/shabal_avx2.c")
        .compile("shabal_avx2.a");
}
