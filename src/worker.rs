use chan;
use futures::sync::mpsc;
use futures::{Future, Sink};
use libc::{c_void, uint64_t};
use miner::Buffer;
use ocl;
use reader::ReadReply;
use std::u64;
extern "C" {
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

pub struct NonceData {
    pub height: u64,
    pub deadline: u64,
    pub nonce: u64,
    pub reader_task_processed: bool,
}

pub fn create_worker_task(
    rx_read_replies: chan::Receiver<ReadReply>,
    tx_empty_buffers: chan::Sender<Box<Buffer + Send>>,
    tx_nonce_data: mpsc::Sender<NonceData>,
) -> impl FnOnce() {
    move || {
        for read_reply in rx_read_replies {
            let buffer = read_reply.buffer;
            if read_reply.len == 0 {
                tx_empty_buffers.send(buffer);
                continue;
            }
            let mut_bs = &*buffer.get_buffer();
            let mut bs = mut_bs.lock().unwrap();
            let gpu_context = buffer.get_context();


            let mut deadline: u64 = u64::MAX;
            let mut offset: u64 = 0;
            
            match &gpu_context {
                None => {
                    let padded = pad(&mut bs, read_reply.len, 8 * 64);
                    unsafe {
                        if is_x86_feature_detected!("avx2") {
                            find_best_deadline_avx2(
                                bs.as_ptr() as *mut c_void,
                                (read_reply.len as u64 + padded as u64) / 64,
                                read_reply.gensig.as_ptr() as *const c_void,
                                &mut deadline,
                                &mut offset,
                            );
                        } else if is_x86_feature_detected!("avx") {
                            find_best_deadline_avx(
                                bs.as_ptr() as *mut c_void,
                                (read_reply.len as u64 + padded as u64) / 64,
                                read_reply.gensig.as_ptr() as *const c_void,
                                &mut deadline,
                                &mut offset,
                            );
                        } else {
                            find_best_deadline_sse2(
                                bs.as_ptr() as *mut c_void,
                                (read_reply.len as u64 + padded as u64) / 64,
                                read_reply.gensig.as_ptr() as *const c_void,
                                &mut deadline,
                                &mut offset,
                            );
                        }
                    }
                }
                Some(context) => {
                    let tuple = ocl::find_best_deadline_gpu(
                        context,
                        bs.as_ptr() as *const c_void,
                        read_reply.len / 64,
                        *read_reply.gensig,
                    );
                    deadline = tuple.0;
                    offset = tuple.1;
                }
            }

            tx_nonce_data
                .clone()
                .send(NonceData {
                    height: read_reply.height,
                    deadline,
                    nonce: offset + read_reply.start_nonce,
                    reader_task_processed: read_reply.finished,
                }).wait()
                .expect("failed to send nonce data");
            tx_empty_buffers.send(buffer);
        }
    }
}

pub fn pad(b: &mut [u8], l: usize, p: usize) -> usize {
    let r = p - l % p;
    if r != p {
        for i in 0..r {
            b[i] = b[0];
        }
        r
    } else {
        0
    }
}
