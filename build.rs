extern crate cc;

fn main() {
    let mut config = cc::Build::new();

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/mshabal_128.c")
        .file("src/c/mshabal_256.c")
        .file("src/c/shabal.c")
        .flag("-mavx2")
        .flag("-std=c99")
        .flag("-march=native")
        .compile("libshabal.a");
}
