extern crate rayon;

use chan;
use filetime::FileTime;
use plot::{Plot, SCOOP_SIZE};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

pub struct ReadReply {
    pub buffer: Arc<Mutex<Vec<u8>>>,
    pub len: usize,
    pub height: u64,
    pub gensig: Arc<[u8; 32]>,
    pub start_nonce: u64,
    pub finished: bool,
}

pub struct Reader {
    drive_id_to_plots: HashMap<String, Arc<Mutex<Vec<RefCell<Plot>>>>>,
    pool: rayon::ThreadPool,
    rx_empty_buffers: chan::Receiver<Arc<Mutex<Vec<u8>>>>,
    tx_read_replies: chan::Sender<ReadReply>,
    interupts: Vec<Sender<()>>,
}

impl Reader {
    pub fn new(
        drive_id_to_plots: HashMap<String, Arc<Mutex<Vec<RefCell<Plot>>>>>,
        num_threads: usize,
        rx_empty_buffers: chan::Receiver<Arc<Mutex<Vec<u8>>>>,
        tx_read_replies: chan::Sender<ReadReply>,
    ) -> Reader {
        for (_, plots) in &drive_id_to_plots {
            let mut plots = plots.lock().unwrap();
            plots.sort_by_key(|p| {
                let m = p.borrow().fh.metadata().unwrap();
                -FileTime::from_last_modification_time(&m).unix_seconds()
            });
        }

        Reader {
            drive_id_to_plots: drive_id_to_plots,
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap(),
            rx_empty_buffers: rx_empty_buffers,
            tx_read_replies: tx_read_replies,
            interupts: Vec::new(),
        }
    }

    pub fn start_reading(&mut self, height: u64, scoop: u32, gensig: Arc<[u8; 32]>) {
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
            })
            .collect();
    }

    fn create_read_task(
        &self,
        plots: Arc<Mutex<Vec<RefCell<Plot>>>>,
        height: u64,
        scoop: u32,
        gensig: Arc<[u8; 32]>,
    ) -> (Sender<()>, impl FnOnce()) {
        let (tx_interupt, rx_interupt) = channel();
        let rx_empty_buffers = self.rx_empty_buffers.clone();
        let tx_read_replies = self.tx_read_replies.clone();

        (tx_interupt, move || {
            let plots = plots.lock().unwrap();
            let plot_count = plots.len();
            'outer: for (i_p, p) in plots.iter().enumerate() {
                let mut p = p.borrow_mut();
                p.read_offset = 0;
                let nonces = p.nonces;
                p.fh
                    .seek(SeekFrom::Start(scoop as u64 * nonces as u64 * SCOOP_SIZE))
                    .unwrap();

                'inner: for buffer in rx_empty_buffers.clone() {
                    let mut bs = buffer.lock().unwrap();

                    let read_offset = p.read_offset;

                    let buffer_cap = bs.capacity();
                    let bytes_to_read =
                        if read_offset as usize + buffer_cap > (SCOOP_SIZE * p.nonces) as usize {
                            (SCOOP_SIZE * p.nonces) as usize - p.read_offset as usize
                        } else {
                            buffer_cap as usize
                        };

                    p.fh
                        .read_exact(&mut bs[0..bytes_to_read])
                        .expect("failed to read chunk");

                    p.read_offset += bytes_to_read as u64;
                    let next_plot = p.read_offset >= SCOOP_SIZE * p.nonces;
                    let finished = i_p == (plot_count - 1) && next_plot;

                    tx_read_replies.send(ReadReply {
                        buffer: buffer.clone(),
                        len: bytes_to_read,
                        height: height,
                        gensig: gensig.clone(),
                        start_nonce: p.start_nonce + (read_offset / 64) as u64,
                        finished: finished,
                    });

                    if next_plot {
                        break 'inner;
                    } else {
                        let offset = p.read_offset;
                        let nonces = p.nonces;
                        p.fh
                            .seek(SeekFrom::Start(
                                offset as u64 + scoop as u64 * nonces as u64 * SCOOP_SIZE,
                            ))
                            .unwrap();
                    }

                    if rx_interupt.try_recv() != Err(TryRecvError::Empty) {
                        break 'outer;
                    }
                }
            }
        })
    }
}
