extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/c/shabal64.s")
        .file("src/c/mshabal_128.c")
        .file("src/c/mshabal_256.c")
        .file("src/c/shabal.c")
        .flag("-mavx2")
        .flag("-std=c99")
        .compile("libshabal.a");
}
