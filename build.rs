extern crate cc;

fn main() {
    let mut config = cc::Build::new();

    #[cfg(target_env = "msvc")]
    config.flag("/arch:AVX2");
    #[cfg(not(target_env = "msvc"))]
    config.flag("-mavx2").flag("-std=c99").flag("-march=native");

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/mshabal_128.c")
        .file("src/c/mshabal_256.c")
        .file("src/c/shabal.c")
        .compile("libshabal.a");
}
