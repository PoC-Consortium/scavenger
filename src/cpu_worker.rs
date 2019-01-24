use chan;
use futures::sync::mpsc;
use futures::{Future, Sink};
use libc::{c_void, uint64_t};
use miner::{Buffer, NonceData};
use reader::ReadReply;
use std::u64;

extern "C" {
    pub fn find_best_deadline_sph(
        scoops: *mut c_void,
        nonce_count: uint64_t,
        gensig: *const c_void,
        best_deadline: *mut uint64_t,
        best_offset: *mut uint64_t,
    ) -> ();
}

cfg_if! {
    if #[cfg(feature = "simd")] {
        extern "C" {
            pub fn find_best_deadline_avx512f(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn find_best_deadline_avx2(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn find_best_deadline_avx(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();

            pub fn find_best_deadline_sse2(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();
        }
    }
}

cfg_if! {
    if #[cfg(feature = "neon")] {
        extern "C" {
            pub fn find_best_deadline_neon(
                scoops: *mut c_void,
                nonce_count: uint64_t,
                gensig: *const c_void,
                best_deadline: *mut uint64_t,
                best_offset: *mut uint64_t,
            ) -> ();
        }
    }
}

pub fn create_cpu_worker_task(
    benchmark: bool,
    thread_pool: rayon::ThreadPool,
    rx_read_replies: chan::Receiver<ReadReply>,
    tx_empty_buffers: chan::Sender<Box<Buffer + Send>>,
    tx_nonce_data: mpsc::Sender<NonceData>,
) -> impl FnOnce() {
    move || {
        for read_reply in rx_read_replies {
            let task = hash(
                read_reply,
                tx_empty_buffers.clone(),
                tx_nonce_data.clone(),
                benchmark,
            );

            thread_pool.spawn(task);
        }
    }
}

pub fn hash(
    read_reply: ReadReply,
    tx_empty_buffers: chan::Sender<Box<Buffer + Send>>,
    tx_nonce_data: mpsc::Sender<NonceData>,
    benchmark: bool,
) -> impl FnOnce() {
    move || {
        let mut buffer = read_reply.buffer;
        // handle empty buffers (read errors) && benchmark
        if read_reply.info.len == 0 || benchmark {
            // forward 'drive finished signal'
            if read_reply.info.finished {
                let deadline = u64::MAX;
                tx_nonce_data
                    .clone()
                    .send(NonceData {
                        height: read_reply.info.height,
                        base_target: read_reply.info.base_target,
                        deadline,
                        nonce: 0,
                        reader_task_processed: read_reply.info.finished,
                        account_id: read_reply.info.account_id,
                    })
                    .wait()
                    .expect("CPU worker failed to send nonce data");
            }
            tx_empty_buffers
                .send(buffer)
                .expect("CPU worker failed to pass through buffer.");
            return;
        }

        // ignore signals
        if read_reply.info.len == 1 && read_reply.info.gpu_signal > 0 {
            return;
        }

        let mut deadline: u64 = u64::MAX;
        let mut offset: u64 = 0;

        let bs = buffer.get_buffer_for_writing();
        let bs = bs.lock().unwrap();

        #[cfg(feature = "simd")]
        unsafe {
            if is_x86_feature_detected!("avx512f") {
                find_best_deadline_avx512f(
                    bs.as_ptr() as *mut c_void,
                    (read_reply.info.len as u64) / 64,
                    read_reply.info.gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            } else if is_x86_feature_detected!("avx2") {
                find_best_deadline_avx2(
                    bs.as_ptr() as *mut c_void,
                    (read_reply.info.len as u64) / 64,
                    read_reply.info.gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            } else if is_x86_feature_detected!("avx") {
                find_best_deadline_avx(
                    bs.as_ptr() as *mut c_void,
                    (read_reply.info.len as u64) / 64,
                    read_reply.info.gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            } else if is_x86_feature_detected!("sse2") {
                find_best_deadline_sse2(
                    bs.as_ptr() as *mut c_void,
                    (read_reply.info.len as u64) / 64,
                    read_reply.info.gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            } else {
                find_best_deadline_sph(
                    bs.as_ptr() as *mut c_void,
                    (read_reply.info.len as u64) / 64,
                    read_reply.info.gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            }
        }
        #[cfg(feature = "neon")]
        unsafe {
            #[cfg(target_arch = "arm")]
            let neon = is_arm_feature_detected!("neon");
            #[cfg(target_arch = "aarch64")]
            let neon = true;
            if neon {
                find_best_deadline_neon(
                    bs.as_ptr() as *mut c_void,
                    (read_reply.info.len as u64) / 64,
                    read_reply.info.gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            } else {
                find_best_deadline_sph(
                    bs.as_ptr() as *mut c_void,
                    (read_reply.info.len as u64) / 64,
                    read_reply.info.gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            }
        }
        #[cfg(not(any(feature = "simd", feature = "neon")))]
        unsafe {
            find_best_deadline_sph(
                bs.as_ptr() as *mut c_void,
                (read_reply.info.len as u64) / 64,
                read_reply.info.gensig.as_ptr() as *const c_void,
                &mut deadline,
                &mut offset,
            );
        }

        tx_nonce_data
            .clone()
            .send(NonceData {
                height: read_reply.info.height,
                base_target: read_reply.info.base_target,
                deadline,
                nonce: offset + read_reply.info.start_nonce,
                reader_task_processed: read_reply.info.finished,
                account_id: read_reply.info.account_id,
            })
            .wait()
            .expect("CPU worker failed to send nonce data");
        tx_empty_buffers
            .send(buffer)
            .expect("CPU worker failed to cue empty buffer");
    }
}

#[cfg(test)]
mod tests {
    use hex;
    use libc::{c_void, uint64_t};
    use std::u64;

    extern "C" {
        pub fn find_best_deadline_sph(
            scoops: *mut c_void,
            nonce_count: uint64_t,
            gensig: *const c_void,
            best_deadline: *mut uint64_t,
            best_offset: *mut uint64_t,
        ) -> ();
    }

    cfg_if! {
        if #[cfg(feature = "simd")] {
            extern "C" {
                pub fn init_shabal_avx512f() -> ();
                pub fn init_shabal_avx2() -> ();
                pub fn init_shabal_avx() -> ();
                pub fn init_shabal_sse2() -> ();
                pub fn find_best_deadline_avx512f(
                    scoops: *mut c_void,
                    nonce_count: uint64_t,
                    gensig: *const c_void,
                    best_deadline: *mut uint64_t,
                    best_offset: *mut uint64_t,
                ) -> ();

                pub fn find_best_deadline_avx2(
                    scoops: *mut c_void,
                    nonce_count: uint64_t,
                    gensig: *const c_void,
                    best_deadline: *mut uint64_t,
                    best_offset: *mut uint64_t,
                ) -> ();

                pub fn find_best_deadline_avx(
                    scoops: *mut c_void,
                    nonce_count: uint64_t,
                    gensig: *const c_void,
                    best_deadline: *mut uint64_t,
                    best_offset: *mut uint64_t,
                ) -> ();

                pub fn find_best_deadline_sse2(
                    scoops: *mut c_void,
                    nonce_count: uint64_t,
                    gensig: *const c_void,
                    best_deadline: *mut uint64_t,
                    best_offset: *mut uint64_t,
                ) -> ();
            }
        }
    }

    cfg_if! {
    if #[cfg(feature = "neon")] {
        extern "C" {
            pub fn init_shabal_neon() -> ();
            pub fn find_best_deadline_neon(
                    scoops: *mut c_void,
                    nonce_count: uint64_t,
                    gensig: *const c_void,
                    best_deadline: *mut uint64_t,
                    best_offset: *mut uint64_t,
                ) -> ();
        }
    }
    }

    #[test]
    fn test_deadline_hashing() {
        let mut deadline: u64 = u64::MAX;
        let mut offset: u64 = 0;
        let gensig =
            hex::decode("4a6f686e6e7946464d206861742064656e206772f6df74656e2050656e697321")
                .unwrap();
        let winner: [u8; 64] = [0; 64];
        let loser: [u8; 64] = [5; 64];
        let mut data: [u8; 64 * 32] = [5; 64 * 32];

        for i in 0..32 {
            data[i * 64..i * 64 + 64].clone_from_slice(&winner);

            unsafe {
                find_best_deadline_sph(
                    data.as_ptr() as *mut c_void,
                    (i + 1) as u64,
                    gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
            }
            assert_eq!(3084580316385335914u64, deadline);
            deadline = u64::MAX;
            offset = 0;
            data[i * 64..i * 64 + 64].clone_from_slice(&loser);
        }
    }

    #[test]
    #[cfg(feature = "simd")]
    fn test_simd_deadline_hashing() {
        let mut deadline: u64 = u64::MAX;
        let mut offset: u64 = 0;
        let gensig =
            hex::decode("4a6f686e6e7946464d206861742064656e206772f6df74656e2050656e697321")
                .unwrap();
        let winner: [u8; 64] = [0; 64];
        let loser: [u8; 64] = [5; 64];
        let mut data: [u8; 64 * 32] = [5; 64 * 32];
        for i in 0..32 {
            data[i * 64..i * 64 + 64].clone_from_slice(&winner);
            unsafe {
                if is_x86_feature_detected!("avx512f") {
                    init_shabal_avx512f();
                    find_best_deadline_avx512f(
                        data.as_ptr() as *mut c_void,
                        (i + 1) as u64,
                        gensig.as_ptr() as *const c_void,
                        &mut deadline,
                        &mut offset,
                    );
                    assert_eq!(3084580316385335914u64, deadline);
                    deadline = u64::MAX;
                    offset = 0;
                }
                if is_x86_feature_detected!("avx2") {
                    init_shabal_avx2();
                    find_best_deadline_avx2(
                        data.as_ptr() as *mut c_void,
                        (i + 1) as u64,
                        gensig.as_ptr() as *const c_void,
                        &mut deadline,
                        &mut offset,
                    );
                    assert_eq!(3084580316385335914u64, deadline);
                    deadline = u64::MAX;
                    offset = 0;
                }
                if is_x86_feature_detected!("avx") {
                    init_shabal_avx();
                    find_best_deadline_avx(
                        data.as_ptr() as *mut c_void,
                        (i + 1) as u64,
                        gensig.as_ptr() as *const c_void,
                        &mut deadline,
                        &mut offset,
                    );
                    assert_eq!(3084580316385335914u64, deadline);
                    deadline = u64::MAX;
                    offset = 0;
                }
                if is_x86_feature_detected!("sse2") {
                    init_shabal_sse2();
                    find_best_deadline_sse2(
                        data.as_ptr() as *mut c_void,
                        (i + 1) as u64,
                        gensig.as_ptr() as *const c_void,
                        &mut deadline,
                        &mut offset,
                    );
                    assert_eq!(3084580316385335914u64, deadline);
                    deadline = u64::MAX;
                    offset = 0;
                }
                find_best_deadline_sph(
                    data.as_ptr() as *mut c_void,
                    (i + 1) as u64,
                    gensig.as_ptr() as *const c_void,
                    &mut deadline,
                    &mut offset,
                );
                assert_eq!(3084580316385335914u64, deadline);
                deadline = u64::MAX;
                offset = 0;
            }
            data[i * 64..i * 64 + 64].clone_from_slice(&loser);
        }
    }
    #[test]
    #[cfg(feature = "neon")]
    fn test_simd_deadline_hashing() {
        let mut deadline: u64 = u64::MAX;
        let mut offset: u64 = 0;
        let gensig =
            hex::decode("4a6f686e6e7946464d206861742064656e206772f6df74656e2050656e697321")
                .unwrap();
        let winner: [u8; 64] = [0; 64];
        let loser: [u8; 64] = [5; 64];
        let mut data: [u8; 64 * 32] = [5; 64 * 32];
        #[cfg(target_arch = "arm")]
        let neon = is_arm_feature_detected!("neon");
        #[cfg(target_arch = "aarch64")]
        let neon = true;
        if neon {
            for i in 0..32 {
                data[i * 64..i * 64 + 64].clone_from_slice(&winner);
                unsafe {
                    init_shabal_neon();
                    find_best_deadline_neon(
                        data.as_ptr() as *mut c_void,
                        (i + 1) as u64,
                        gensig.as_ptr() as *const c_void,
                        &mut deadline,
                        &mut offset,
                    );
                }
                assert_eq!(3084580316385335914u64, deadline);
                data[i * 64..i * 64 + 64].clone_from_slice(&loser);
                deadline = u64::MAX;
                offset = 0;
            }
        }
    }
}
