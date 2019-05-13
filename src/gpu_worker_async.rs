use crate::miner::{Buffer, NonceData};
use crate::ocl::GpuContext;
use crate::ocl::{gpu_hash, gpu_transfer, gpu_transfer_and_hash};
use crate::reader::{BufferInfo, ReadReply};
use crossbeam_channel::{Receiver, Sender};
use futures::sync::mpsc;
use futures::{Future, Sink};
use std::sync::Arc;
use std::u64;

pub fn create_gpu_worker_task_async(
    benchmark: bool,
    rx_read_replies: Receiver<ReadReply>,
    tx_empty_buffers: Sender<Box<Buffer + Send>>,
    tx_nonce_data: mpsc::Sender<NonceData>,
    context_mu: Arc<GpuContext>,
    num_drives: usize,
) -> impl FnOnce() {
    move || {
        let mut new_round = true;
        let mut last_buffer_a = None;
        let mut last_buffer_info_a = BufferInfo {
            len: 0,
            height: 0,
            block: 0,
            base_target: 0,
            gensig: Arc::new([0u8; 32]),
            start_nonce: 0,
            finished: false,
            account_id: 0,
            gpu_signal: 0,
        };
        let mut drive_count = 0;
        let (tx_sink, rx_sink) = crossbeam_channel::bounded(1);
        let mut active_height = 0;
        for read_reply in rx_read_replies {
            let buffer = read_reply.buffer;
            // handle empty buffers (read errors) && benchmark
            if read_reply.info.len == 0 || benchmark {
                // forward 'drive finished signal'
                if read_reply.info.finished {
                    let deadline = u64::MAX;
                    tx_nonce_data
                        .clone()
                        .send(NonceData {
                            height: read_reply.info.height,
                            block: read_reply.info.block,
                            base_target: read_reply.info.base_target,
                            deadline,
                            nonce: 0,
                            reader_task_processed: read_reply.info.finished,
                            account_id: read_reply.info.account_id,
                        })
                        .wait()
                        .expect("GPU async worker failed to send nonce data");
                }
                tx_empty_buffers
                    .send(buffer)
                    .expect("GPU async worker failed to cue empty buffer");
                continue;
            }

            // process start signal
            if read_reply.info.gpu_signal == 1 {
                if !new_round {
                    if let Ok(sink_buffer) = rx_sink.try_recv() {
                        tx_empty_buffers
                            .send(sink_buffer)
                            .expect("GPU async worker failed to cue empty buffer from sink")
                    }
                }
                drive_count = 0;
                active_height = read_reply.info.height;
                new_round = true;
                continue;
            }

            // end signal
            if read_reply.info.gpu_signal == 2 && active_height == read_reply.info.height {
                drive_count += 1;
                if drive_count == num_drives && !new_round {
                    let result = gpu_hash(
                        &context_mu,
                        last_buffer_info_a.len / 64,
                        last_buffer_a.as_ref().unwrap(),
                    );
                    let deadline = result.0;
                    let offset = result.1;

                    tx_nonce_data
                        .clone()
                        .send(NonceData {
                            height: last_buffer_info_a.height,
                            block: last_buffer_info_a.block,
                            base_target: last_buffer_info_a.base_target,
                            deadline,
                            nonce: offset + last_buffer_info_a.start_nonce,
                            reader_task_processed: last_buffer_info_a.finished,
                            account_id: last_buffer_info_a.account_id,
                        })
                        .wait()
                        .expect("GPU async worker failed to send nonce data");
                    if let Ok(sink_buffer) = rx_sink.try_recv() {
                        tx_empty_buffers
                            .send(sink_buffer)
                            .expect("GPU async worker failed to cue empty buffer from sink")
                    }
                }
                continue;
            }
            if read_reply.info.gpu_signal == 2 {
                continue;
            }

            if new_round {
                gpu_transfer(
                    &context_mu,
                    buffer.get_gpu_buffers().unwrap(),
                    *read_reply.info.gensig,
                );
            } else {
                let result = gpu_transfer_and_hash(
                    &context_mu,
                    buffer.get_gpu_buffers().unwrap(),
                    last_buffer_info_a.len / 64,
                    last_buffer_a.as_ref().unwrap(),
                );
                let deadline = result.0;
                let offset = result.1;

                tx_nonce_data
                    .clone()
                    .send(NonceData {
                        height: last_buffer_info_a.height,
                        block: last_buffer_info_a.block,
                        base_target: last_buffer_info_a.base_target,
                        deadline,
                        nonce: offset + last_buffer_info_a.start_nonce,
                        reader_task_processed: last_buffer_info_a.finished,
                        account_id: last_buffer_info_a.account_id,
                    })
                    .wait()
                    .expect("GPU async worker failed to cue empty buffer");
                if let Ok(sink_buffer) = rx_sink.try_recv() {
                    tx_empty_buffers
                        .send(sink_buffer)
                        .expect("GPU async worker failed to cue empty buffer from sink")
                }
            }
            last_buffer_a = buffer.get_gpu_data();
            last_buffer_info_a = read_reply.info;
            new_round = false;
            tx_sink
                .send(buffer)
                .expect("GPU async worker failed to cue buffer in sink");
        }
    }
}
