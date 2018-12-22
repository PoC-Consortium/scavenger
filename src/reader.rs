extern crate pbr;
extern crate rayon;

use self::pbr::{ProgressBar, Units};
use chan;
use core_affinity;
use filetime::FileTime;
use miner::Buffer;
use plot::Plot;
use reader::rayon::prelude::*;
use std::collections::HashMap;
use std::io::Stdout;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use stopwatch::Stopwatch;
#[cfg(windows)]
use utils::set_thread_ideal_processor;

pub struct ReadReply {
    pub buffer: Box<Buffer + Send>,
    pub len: usize,
    pub height: u64,
    pub gensig: Arc<[u8; 32]>,
    pub start_nonce: u64,
    pub finished: bool,
    pub account_id: u64,
}

pub struct Reader {
    drive_id_to_plots: HashMap<String, Arc<Mutex<Vec<RwLock<Plot>>>>>,
    pub total_size: u64,
    pool: rayon::ThreadPool,
    rx_empty_buffers: chan::Receiver<Box<Buffer + Send>>,
    tx_read_replies_cpu: chan::Sender<ReadReply>,
    tx_read_replies_gpu: chan::Sender<ReadReply>,
    interupts: Vec<Sender<()>>,
    show_progress: bool,
    show_drive_stats: bool,
}

impl Reader {
    pub fn new(
        drive_id_to_plots: HashMap<String, Arc<Mutex<Vec<RwLock<Plot>>>>>,
        total_size: u64,
        num_threads: usize,
        rx_empty_buffers: chan::Receiver<Box<Buffer + Send>>,
        tx_read_replies_cpu: chan::Sender<ReadReply>,
        tx_read_replies_gpu: chan::Sender<ReadReply>,
        show_progress: bool,
        show_drive_stats: bool,
        thread_pinning: bool,
    ) -> Reader {
        for plots in drive_id_to_plots.values() {
            let mut plots = plots.lock().unwrap();
            plots.sort_by_key(|p| {
                let m = p.read().unwrap().fh.metadata().unwrap();
                -FileTime::from_last_modification_time(&m).unix_seconds()
            });
        }

        check_overlap(&drive_id_to_plots);

        let mut core_ids: Vec<core_affinity::CoreId> = Vec::new();
        if thread_pinning {
            core_ids = core_affinity::get_core_ids().unwrap();
        }

        Reader {
            drive_id_to_plots,
            total_size,
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .start_handler(move |id| {
                    if thread_pinning {
                        #[cfg(not(windows))]
                        let core_id = core_ids[id % core_ids.len()];
                        #[cfg(not(windows))]
                        core_affinity::set_for_current(core_id);
                        #[cfg(windows)]
                        set_thread_ideal_processor(id % core_ids.len());
                    }
                })
                .build()
                .unwrap(),
            rx_empty_buffers,
            tx_read_replies_cpu,
            tx_read_replies_gpu,
            interupts: Vec::new(),
            show_progress,
            show_drive_stats,
        }
    }

    pub fn start_reading(&mut self, height: u64, scoop: u32, gensig: &Arc<[u8; 32]>) {
        for interupt in &self.interupts {
            interupt.send(()).ok();
        }

        let mut pb = ProgressBar::new(self.total_size);
        pb.format("│██░│");
        pb.set_width(Some(80));
        pb.set_units(Units::Bytes);
        pb.message("Scavenging: ");
        let pb = Arc::new(Mutex::new(pb));

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
                let plots = plots.lock().unwrap();
                let mut p = plots[0].write().unwrap();

                if let Err(e) = p.seek_random() {
                    error!(
                        "wakeup: error during wakeup {}: {} -> skip one round",
                        p.name, e
                    );
                }
            });
        }
    }

    fn create_read_task(
        &self,
        pb: Option<Arc<Mutex<pbr::ProgressBar<Stdout>>>>,
        drive: String,
        plots: Arc<Mutex<Vec<RwLock<Plot>>>>,
        height: u64,
        scoop: u32,
        gensig: Arc<[u8; 32]>,
        show_drive_stats: bool,
    ) -> (Sender<()>, impl FnOnce()) {
        //Pin!

        let (tx_interupt, rx_interupt) = channel();
        let rx_empty_buffers = self.rx_empty_buffers.clone();
        let tx_read_replies_cpu = self.tx_read_replies_cpu.clone();
        #[cfg(feature = "opencl")]
        let tx_read_replies_gpu = self.tx_read_replies_gpu.clone();
        #[cfg(not(feature = "opencl"))]
        let _tx_read_replies_gpu = self.tx_read_replies_gpu.clone();
        (tx_interupt, move || {
            let mut sw = Stopwatch::new();
            let mut elapsed = 0i64;
            let mut nonces_processed = 0u64;
            let plots = plots.lock().unwrap();
            let plot_count = plots.len();
            'outer: for (i_p, p) in plots.iter().enumerate() {
                let mut p = p.write().unwrap();
                if let Err(e) = p.prepare(scoop) {
                    error!(
                        "reader: error preparing {} for reading: {} -> skip one round",
                        p.name, e
                    );
                    continue 'outer;
                }

                'inner: for mut buffer in rx_empty_buffers.clone() {
                    if show_drive_stats {
                        sw.restart();
                    }
                    let mut_bs = &*buffer.get_buffer_for_writing();
                    let mut bs = mut_bs.lock().unwrap();
                    let (bytes_read, start_nonce, next_plot) = match p.read(&mut *bs, scoop) {
                        Ok(x) => x,
                        Err(e) => {
                            error!(
                                "reader: error reading chunk from {}: {} -> skip one round",
                                p.name, e
                            );
                            (0, 0, true)
                        }
                    };

                    let finished = i_p == (plot_count - 1) && next_plot;
                    //fork

                    #[cfg(feature = "opencl")]
                    let gpu_context = buffer.get_gpu_context();
                    #[cfg(feature = "opencl")]
                    match &gpu_context {
                        None => {
                            tx_read_replies_cpu.send(ReadReply {
                                buffer,
                                len: bytes_read,
                                height,
                                gensig: gensig.clone(),
                                start_nonce,
                                finished,
                                account_id: p.account_id,
                            });
                        }
                        Some(_context) => {
                            tx_read_replies_gpu.send(ReadReply {
                                buffer,
                                len: bytes_read,
                                height,
                                gensig: gensig.clone(),
                                start_nonce,
                                finished,
                                account_id: p.account_id,
                            });
                        }
                    }
                    #[cfg(not(feature = "opencl"))]
                    tx_read_replies_cpu.send(ReadReply {
                        buffer,
                        len: bytes_read,
                        height,
                        gensig: gensig.clone(),
                        start_nonce,
                        finished,
                        account_id: p.account_id,
                    });

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
                    if rx_interupt.try_recv() != Err(TryRecvError::Empty) {
                        break 'outer;
                    }
                }
            }
        })
    }
}

// Don't waste your time striving for perfection; instead, strive for excellence - doing your best.
// let my_best = perfection;
pub fn check_overlap(drive_id_to_plots: &HashMap<String, Arc<Mutex<Vec<RwLock<Plot>>>>>) -> bool {
    let mut result = false;
    for (i, drive_a) in drive_id_to_plots.values().enumerate() {
        for (j, drive_b) in drive_id_to_plots.values().skip(i).enumerate() {
            if i == j + i {
                let drive = drive_a.lock().unwrap();
                let dupes = drive.par_iter().enumerate().filter(|(x, j)| {
                    drive
                        .par_iter()
                        .skip(x + 1)
                        .filter(|l| {
                            let plot_a = l.write().unwrap();
                            let plot_b = j.write().unwrap();
                            plot_a.account_id == plot_b.account_id && plot_a.overlaps_with(&plot_b)
                        })
                        .count()
                        > 0
                });
                result |= dupes.count() > 0;
            } else {
                let drive_a = drive_a.lock().unwrap();
                let drive_b = drive_b.lock().unwrap();
                let dupes = drive_a.par_iter().filter(|j| {
                    drive_b
                        .par_iter()
                        .filter(|l| {
                            let plot_a = l.write().unwrap();
                            let plot_b = j.write().unwrap();
                            plot_a.account_id == plot_b.account_id && plot_a.overlaps_with(&plot_b)
                        })
                        .count()
                        > 0
                });
                result |= dupes.count() > 0;
            }
        }
    }
    result
}
