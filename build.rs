extern crate cc;
#[macro_use]
extern crate cfg_if;

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
    shared_config.flag("-std=c99");

    #[cfg(not(target_env = "msvc"))]
    shared_config.flag("-mtune=native");

    let mut config = shared_config.clone();

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/shabal.c")
        .file("src/c/common.c")
        .compile("shabal");

    cfg_if! {
         if #[cfg(feature = "neon")] {
             fn build(shared_config: cc::Build){
                let mut config = shared_config.clone();

                #[cfg(all(not(target_env = "msvc"), not(target_arch = "aarch64")))]
                config.flag("-mfpu=neon");

                config
                    .file("src/c/mshabal_128_neon.c")
                    .file("src/c/shabal_neon.c")
                    .compile("shabal_neon");
             }
         }
    }

    cfg_if! {
         if #[cfg(feature = "simd")] {
             fn build(shared_config: cc::Build){
                let mut config = shared_config.clone();

                #[cfg(not(target_env = "msvc"))]
                config.flag("-msse2");

                config
                    .file("src/c/mshabal_128_sse2.c")
                    .file("src/c/shabal_sse2.c")
                    .compile("shabal_sse2");

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
            }
        }
    }
    #[cfg(any(feature = "simd", feature = "neon"))]
    build(shared_config);
}
