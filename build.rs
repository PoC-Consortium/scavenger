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
        .flag("/GL")
        .flag("/arch:AVX2");
    #[cfg(not(target_env = "msvc"))]
    config.flag("-std=c99").flag("-march=native");

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/mshabal_128.c");

    if is_x86_feature_detected!("avx") {
        #[cfg(not(target_env = "msvc"))]
        config.flag("-mavx2");

        config.file("src/c/mshabal_256.c").file("src/c/shabal.c");
    } else {
        config.file("src/c/shabal_sse2.c");
    }

    config.flag("-O2"); // -O3 fails on xeon and maybe others
    config.compile("libshabal.a");
}
