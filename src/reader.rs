use crate::miner::Buffer;
#[cfg(feature = "opencl")]
use crate::miner::CpuBuffer;
use crate::plot::{Meta, Plot};
use crate::utils::new_thread_pool;
use crossbeam_channel;
use crossbeam_channel::{Receiver, Sender};
use pbr::{ProgressBar, Units};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Stdout;
use std::sync::{Arc, Mutex};
use stopwatch::Stopwatch;

pub struct BufferInfo {
    pub len: usize,
    pub height: u64,
    pub block: u64,
    pub base_target: u64,
    pub gensig: Arc<[u8; 32]>,
    pub start_nonce: u64,
    pub finished: bool,
    pub account_id: u64,
    pub gpu_signal: u64,
}
pub struct ReadReply {
    pub buffer: Box<Buffer + Send>,
    pub info: BufferInfo,
}

#[allow(dead_code)]
pub struct Reader {
    drive_id_to_plots: HashMap<String, Arc<Vec<Mutex<Plot>>>>,
    pub total_size: u64,
    pool: rayon::ThreadPool,
    rx_empty_buffers: Receiver<Box<Buffer + Send>>,
    tx_empty_buffers: Sender<Box<Buffer + Send>>,
    tx_read_replies_cpu: Sender<ReadReply>,
    tx_read_replies_gpu: Option<Vec<Sender<ReadReply>>>,
    interupts: Vec<Sender<()>>,
    show_progress: bool,
    show_drive_stats: bool,
}

impl Reader {
    pub fn new(
        drive_id_to_plots: HashMap<String, Arc<Vec<Mutex<Plot>>>>,
        total_size: u64,
        num_threads: usize,
        rx_empty_buffers: Receiver<Box<Buffer + Send>>,
        tx_empty_buffers: Sender<Box<Buffer + Send>>,
        tx_read_replies_cpu: Sender<ReadReply>,
        tx_read_replies_gpu: Option<Vec<Sender<ReadReply>>>,
        show_progress: bool,
        show_drive_stats: bool,
        thread_pinning: bool,
        benchmark: bool,
    ) -> Reader {
        if !benchmark {
            check_overlap(&drive_id_to_plots);
        }

        Reader {
            drive_id_to_plots,
            total_size,
            pool: new_thread_pool(num_threads, thread_pinning),
            rx_empty_buffers,
            tx_empty_buffers,
            tx_read_replies_cpu,
            tx_read_replies_gpu,
            interupts: Vec::new(),
            show_progress,
            show_drive_stats,
        }
    }

    pub fn start_reading(
        &mut self,
        height: u64,
        block: u64,
        base_target: u64,
        scoop: u32,
        gensig: &Arc<[u8; 32]>,
    ) {
        for interupt in &self.interupts {
            interupt.send(()).ok();
        }
        let mut pb = ProgressBar::new(self.total_size);
        pb.format("│██░│");
        pb.set_width(Some(80));
        pb.set_units(Units::Bytes);
        pb.message("Scavenging: ");
        let pb = Arc::new(Mutex::new(pb));

        // send start signals (dummy buffer) to gpu threads
        #[cfg(feature = "opencl")]
        for i in 0..self.tx_read_replies_gpu.as_ref().unwrap().len() {
            self.tx_read_replies_gpu.as_ref().unwrap()[i]
                .send(ReadReply {
                    buffer: Box::new(CpuBuffer::new(0)) as Box<Buffer + Send>,
                    info: BufferInfo {
                        len: 1,
                        height,
                        block,
                        base_target,
                        gensig: gensig.clone(),
                        start_nonce: 0,
                        finished: false,
                        account_id: 0,
                        gpu_signal: 1,
                    },
                })
                .expect("Error sending 'round start' signal to GPU");
        }

        self.interupts = self
            .drive_id_to_plots
            .iter()
            .map(|(drive, plots)| {
                let (interupt, task) = if self.show_progress {
                    self.create_read_task(
                        Some(pb.clone()),
                        drive.clone(),
                        plots.clone(),
                        height,
                        block,
                        base_target,
                        scoop,
                        gensig.clone(),
                        self.show_drive_stats,
                    )
                } else {
                    self.create_read_task(
                        None,
                        drive.clone(),
                        plots.clone(),
                        height,
                        block,
                        base_target,
                        scoop,
                        gensig.clone(),
                        self.show_drive_stats,
                    )
                };

                self.pool.spawn(task);
                interupt
            })
            .collect();
    }

    pub fn wakeup(&mut self) {
        for plots in self.drive_id_to_plots.values() {
            let plots = plots.clone();
            self.pool.spawn(move || {
                let mut p = plots[0].lock().unwrap();

                if let Err(e) = p.seek_random() {
                    error!(
                        "wakeup: error during wakeup {}: {} -> skip one round",
                        p.meta.name, e
                    );
                }
            });
        }
    }

    fn create_read_task(
        &self,
        pb: Option<Arc<Mutex<pbr::ProgressBar<Stdout>>>>,
        drive: String,
        plots: Arc<Vec<Mutex<Plot>>>,
        height: u64,
        block: u64,
        base_target: u64,
        scoop: u32,
        gensig: Arc<[u8; 32]>,
        show_drive_stats: bool,
    ) -> (Sender<()>, impl FnOnce()) {
        let (tx_interupt, rx_interupt) = crossbeam_channel::unbounded();
        let rx_empty_buffers = self.rx_empty_buffers.clone();
        let tx_empty_buffers = self.tx_empty_buffers.clone();
        let tx_read_replies_cpu = self.tx_read_replies_cpu.clone();
        #[cfg(feature = "opencl")]
        let tx_read_replies_gpu = self.tx_read_replies_gpu.clone();

        (tx_interupt, move || {
            let mut sw = Stopwatch::new();
            let mut elapsed = 0i64;
            let mut nonces_processed = 0u64;
            let plot_count = plots.len();
            'outer: for (i_p, p) in plots.iter().enumerate() {
                let mut p = p.lock().unwrap();
                if let Err(e) = p.prepare(scoop) {
                    error!(
                        "reader: error preparing {} for reading: {} -> skip one round",
                        p.meta.name, e
                    );
                    continue 'outer;
                }

                'inner: for mut buffer in rx_empty_buffers.clone() {
                    if show_drive_stats {
                        sw.restart();
                    }
                    let mut_bs = buffer.get_buffer_for_writing();
                    let mut bs = mut_bs.lock().unwrap();
                    let (bytes_read, start_nonce, next_plot) = match p.read(&mut bs, scoop) {
                        Ok(x) => x,
                        Err(e) => {
                            error!(
                                "reader: error reading chunk from {}: {} -> skip one round",
                                p.meta.name, e
                            );
                            buffer.unmap();
                            (0, 0, true)
                        }
                    };

                    if rx_interupt.try_recv().is_ok() {
                        buffer.unmap();
                        tx_empty_buffers.send(buffer).unwrap();
                        break 'outer;
                    }

                    let finished = i_p == (plot_count - 1) && next_plot;
                    // buffer routing
                    #[cfg(feature = "opencl")]
                    match buffer.get_id() {
                        0 => {
                            tx_read_replies_cpu
                                .send(ReadReply {
                                    buffer,
                                    info: BufferInfo {
                                        len: bytes_read,
                                        height,
                                        block,
                                        base_target,
                                        gensig: gensig.clone(),
                                        start_nonce,
                                        finished,
                                        account_id: p.meta.account_id,
                                        gpu_signal: 0,
                                    },
                                })
                                .expect("failed to send read data to CPU thread");
                        }
                        i => {
                            tx_read_replies_gpu.as_ref().unwrap()[i - 1]
                                .send(ReadReply {
                                    buffer,
                                    info: BufferInfo {
                                        len: bytes_read,
                                        height,
                                        block,
                                        base_target,
                                        gensig: gensig.clone(),
                                        start_nonce,
                                        finished,
                                        account_id: p.meta.account_id,
                                        gpu_signal: 0,
                                    },
                                })
                                .expect("failed to send read data to GPU thread A");
                        }
                    }
                    #[cfg(not(feature = "opencl"))]
                    tx_read_replies_cpu
                        .send(ReadReply {
                            buffer,
                            info: BufferInfo {
                                len: bytes_read,
                                height,
                                block,
                                base_target,
                                gensig: gensig.clone(),
                                start_nonce,
                                finished,
                                account_id: p.meta.account_id,
                                gpu_signal: 0,
                            },
                        })
                        .unwrap();

                    nonces_processed += bytes_read as u64 / 64;

                    match &pb {
                        Some(pb) => {
                            let mut pb = pb.lock().unwrap();
                            pb.add(bytes_read as u64);
                        }
                        None => (),
                    }

                    if show_drive_stats {
                        elapsed += sw.elapsed_ms();
                    }

                    // send termination signal (dummy buffer) to gpu
                    if finished {
                        #[cfg(feature = "opencl")]
                        for i in 0..tx_read_replies_gpu.as_ref().unwrap().len() {
                            tx_read_replies_gpu.as_ref().unwrap()[i]
                                .send(ReadReply {
                                    buffer: Box::new(CpuBuffer::new(0)) as Box<Buffer + Send>,
                                    info: BufferInfo {
                                        len: 1,
                                        height,
                                        block,
                                        base_target,
                                        gensig: gensig.clone(),
                                        start_nonce: 0,
                                        finished: false,
                                        account_id: 0,
                                        gpu_signal: 2,
                                    },
                                })
                                .expect("Error sending 'drive finished' signal to GPU thread A");
                        }
                    }

                    if finished && show_drive_stats {
                        info!(
                            "{: <80}",
                            format!(
                                "drive {} finished, speed={} MiB/s",
                                drive,
                                nonces_processed * 1000 / (elapsed + 1) as u64 * 64 / 1024 / 1024,
                            )
                        );
                    }

                    if next_plot {
                        break 'inner;
                    }
                }
            }
        })
    }
}

// Don't waste your time striving for perfection; instead, strive for excellence - doing your best.
// let my_best = perfection;
pub fn check_overlap(drive_id_to_plots: &HashMap<String, Arc<Vec<Mutex<Plot>>>>) -> bool {
    let plots: Vec<Meta> = drive_id_to_plots
        .values()
        .map(|a| a.iter())
        .flatten()
        .map(|plot| plot.lock().unwrap().meta.clone())
        .collect();
    plots
        .par_iter()
        .enumerate()
        .filter(|(i, plot_a)| {
            plots[i + 1..]
                .par_iter()
                .filter(|plot_b| {
                    plot_a.account_id == plot_b.account_id && plot_b.overlaps_with(&plot_a)
                })
                .count()
                > 0
        })
        .count()
        > 0
}
