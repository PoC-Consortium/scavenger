extern crate rayon;

use chan;
use filetime::FileTime;
use miner::Buffer;
use plot::Plot;
use reader::rayon::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::RwLock;
use std::sync::{Arc, Mutex};

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
    pool: rayon::ThreadPool,
    rx_empty_buffers: chan::Receiver<Box<Buffer + Send>>,
    tx_read_replies_cpu: chan::Sender<ReadReply>,
    tx_read_replies_gpu: chan::Sender<ReadReply>,
    interupts: Vec<Sender<()>>,
}

impl Reader {
    pub fn new(
        drive_id_to_plots: HashMap<String, Arc<Mutex<Vec<RwLock<Plot>>>>>,
        num_threads: usize,
        rx_empty_buffers: chan::Receiver<Box<Buffer + Send>>,
        tx_read_replies_cpu: chan::Sender<ReadReply>,
        tx_read_replies_gpu: chan::Sender<ReadReply>,
    ) -> Reader {
        for plots in drive_id_to_plots.values() {
            let mut plots = plots.lock().unwrap();
            plots.sort_by_key(|p| {
                let m = p.read().unwrap().fh.metadata().unwrap();
                -FileTime::from_last_modification_time(&m).unix_seconds()
            });
        }

        check_overlap(&drive_id_to_plots);

        Reader {
            drive_id_to_plots,
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap(),
            rx_empty_buffers,
            tx_read_replies_cpu,
            tx_read_replies_gpu,
            interupts: Vec::new(),
        }
    }

    pub fn start_reading(&mut self, height: u64, scoop: u32, gensig: &Arc<[u8; 32]>) {
        for interupt in &self.interupts {
            interupt.send(()).ok();
        }
        self.interupts = self
            .drive_id_to_plots
            .iter()
            .map(|(_, plots)| {
                let (interupt, task) =
                    self.create_read_task(plots.clone(), height, scoop, gensig.clone());
                self.pool.spawn(task);
                interupt
            }).collect();
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
        plots: Arc<Mutex<Vec<RwLock<Plot>>>>,
        height: u64,
        scoop: u32,
        gensig: Arc<[u8; 32]>,
    ) -> (Sender<()>, impl FnOnce()) {
        let (tx_interupt, rx_interupt) = channel();
        let rx_empty_buffers = self.rx_empty_buffers.clone();
        let tx_read_replies_cpu = self.tx_read_replies_cpu.clone();
        let tx_read_replies_gpu = self.tx_read_replies_gpu.clone();

        (tx_interupt, move || {
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
                    let gpu_context = buffer.get_gpu_context();

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
                        }).count()
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
                        }).count()
                        > 0
                });
                result |= dupes.count() > 0;
            }
        }
    }
    result
}
