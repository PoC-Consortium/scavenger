extern crate aligned_alloc;
extern crate num_cpus;
#[cfg(feature = "opencl")]
extern crate ocl_core as core;
extern crate page_size;

use burstmath;
use chan;
use config::Cfg;
use core_affinity;
use futures::sync::mpsc;
use plot::{Plot, SCOOP_SIZE};
use reader::Reader;
use requests::RequestHandler;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::read_dir;
use std::path::Path;
use std::rc::Rc;
use std::sync::RwLock;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::u64;
use stopwatch::Stopwatch;
use tokio::prelude::*;
use tokio::timer::Interval;
use tokio_core::reactor::Core;
use utils::get_device_id;
#[cfg(windows)]
use utils::set_thread_ideal_processor;
use worker::{create_worker_task, NonceData};

#[cfg(feature = "opencl")]
use ocl::GpuBuffer;
#[cfg(feature = "opencl")]
use ocl::GpuContext;

pub struct Miner {
    reader: Reader,
    request_handler: RequestHandler,
    rx_nonce_data: mpsc::Receiver<NonceData>,
    target_deadline: u64,
    state: Arc<Mutex<State>>,
    reader_task_count: usize,
    get_mining_info_interval: u64,
    core: Core,
    wakeup_after: i64,
}

pub struct State {
    height: u64,
    account_id_to_best_deadline: HashMap<u64, u64>,
    base_target: u64,
    sw: Stopwatch,
    scanning: bool,

    // count how many reader's scoops have been processed
    processed_reader_tasks: usize,
}

pub trait Buffer {
    fn get_buffer(&mut self) -> Arc<Mutex<Vec<u8>>>;

    fn get_buffer_for_writing(&mut self) -> Arc<Mutex<Vec<u8>>>;
    #[cfg(feature = "opencl")]
    fn get_gpu_context(&self) -> Option<Arc<Mutex<GpuContext>>>;
    #[cfg(feature = "opencl")]
    fn get_gpu_buffers(&self) -> Option<&GpuBuffer>;
}

pub struct CpuBuffer {
    data: Arc<Mutex<Vec<u8>>>,
}

impl CpuBuffer {
    fn new(buffer_size: usize) -> Self
    where
        Self: Sized,
    {
        let pointer = aligned_alloc::aligned_alloc(buffer_size, page_size::get());
        let data: Vec<u8>;
        unsafe {
            data = Vec::from_raw_parts(pointer as *mut u8, buffer_size, buffer_size);
        }
        CpuBuffer {
            data: Arc::new(Mutex::new(data)),
        }
    }
}

impl Buffer for CpuBuffer {
    fn get_buffer(&mut self) -> Arc<Mutex<Vec<u8>>> {
        self.data.clone()
    }
    fn get_buffer_for_writing(&mut self) -> Arc<Mutex<Vec<u8>>> {
        self.data.clone()
    }
    #[cfg(feature = "opencl")]
    fn get_gpu_context(&self) -> Option<Arc<Mutex<GpuContext>>> {
        None
    }
    #[cfg(feature = "opencl")]
    fn get_gpu_buffers(&self) -> Option<&GpuBuffer> {
        None
    }
}

fn scan_plots(
    plot_dirs: &[String],
    use_direct_io: bool,
    dummy: bool,
) -> (HashMap<String, Arc<Mutex<Vec<RwLock<Plot>>>>>, u64) {
    let mut drive_id_to_plots: HashMap<String, Arc<Mutex<Vec<RwLock<Plot>>>>> = HashMap::new();
    let mut global_capacity: u64 = 0;

    for plot_dir_str in plot_dirs {
        let dir = Path::new(plot_dir_str);

        if !dir.exists() {
            warn!("path {} does not exist", plot_dir_str);
            continue;
        }
        if !dir.is_dir() {
            warn!("path {} is not a directory", plot_dir_str);
            continue;
        }

        let mut num_plots = 0;
        let mut local_capacity: u64 = 0;
        for file in read_dir(dir).unwrap() {
            let file = &file.unwrap().path();

            if let Ok(p) = Plot::new(file, use_direct_io, dummy) {
                let drive_id = get_device_id(&file.to_str().unwrap().to_string());
                let plots = drive_id_to_plots
                    .entry(drive_id)
                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

                local_capacity += p.nonces as u64;
                plots.lock().unwrap().push(RwLock::new(p));
                num_plots += 1;
            }
        }

        info!(
            "path={}, files={}, size={:.4} TiB",
            plot_dir_str,
            num_plots,
            local_capacity as f64 / 4.0 / 1024.0 / 1024.0
        );

        global_capacity += local_capacity;
        if num_plots == 0 {
            warn!("no plots in {}", plot_dir_str);
        }
    }

    info!(
        "plot files loaded: total drives={}, total capacity={:.4} TiB",
        drive_id_to_plots.len(),
        global_capacity as f64 / 4.0 / 1024.0 / 1024.0
    );

    (drive_id_to_plots, global_capacity * 64)
}

impl Miner {
    pub fn new(cfg: Cfg) -> Miner {
        let (drive_id_to_plots, total_size) = scan_plots(
            &cfg.plot_dirs,
            cfg.hdd_use_direct_io,
            cfg.benchmark_only.to_uppercase() == "XPU",
        );

        let reader_thread_count = if cfg.hdd_reader_thread_count == 0 {
            drive_id_to_plots.len()
        } else {
            cfg.hdd_reader_thread_count
        };

        let cpu_worker_thread_count = cfg.cpu_worker_thread_count;
        let gpu_worker_thread_count = cfg.gpu_worker_thread_count;

        info!(
            "CPU-worker: {}, GPU-worker: {}",
            cpu_worker_thread_count, gpu_worker_thread_count
        );

        let buffer_count = cpu_worker_thread_count * 2 + gpu_worker_thread_count * 2;
        let buffer_size_cpu = cfg.cpu_nonces_per_cache * SCOOP_SIZE as usize;

        let (tx_empty_buffers, rx_empty_buffers) = chan::bounded(buffer_count as usize);
        let (tx_read_replies_cpu, rx_read_replies_cpu) = chan::bounded(cpu_worker_thread_count * 2);
        let (tx_read_replies_gpu, rx_read_replies_gpu) = chan::bounded(gpu_worker_thread_count * 2);

        #[cfg(feature = "opencl")]
        let mut vec = Vec::new();

        #[cfg(feature = "opencl")]
        for _ in 0..gpu_worker_thread_count {
            vec.push(Arc::new(Mutex::new(GpuContext::new(
                cfg.gpu_platform,
                cfg.gpu_device,
                cfg.gpu_nonces_per_cache,
                if cfg.benchmark_only.to_uppercase() == "I/O" {
                    false
                } else {
                    cfg.gpu_mem_mapping
                },
            ))));
        }

        #[cfg(feature = "opencl")]
        for _ in 0..1 {
            for i in 0..gpu_worker_thread_count {
                let gpu_buffer = GpuBuffer::new(&vec[i]);
                tx_empty_buffers.send(Box::new(gpu_buffer) as Box<Buffer + Send>);
            }
        }

        for _ in 0..cpu_worker_thread_count * 2 {
            let cpu_buffer = CpuBuffer::new(buffer_size_cpu);
            tx_empty_buffers.send(Box::new(cpu_buffer) as Box<Buffer + Send>);
        }

        let (tx_nonce_data, rx_nonce_data) =
            mpsc::channel(cpu_worker_thread_count + gpu_worker_thread_count);

        let core_ids = core_affinity::get_core_ids().unwrap();
        for id in 0..cpu_worker_thread_count {
            thread::spawn({
                if cfg.cpu_thread_pinning {
                    #[cfg(not(windows))]
                    let core_id = core_ids[id % core_ids.len()];
                    #[cfg(not(windows))]
                    core_affinity::set_for_current(core_id);
                    #[cfg(windows)]
                    set_thread_ideal_processor(id % core_ids.len());
                }
                create_worker_task(
                    cfg.benchmark_only.to_uppercase() == "I/O",
                    rx_read_replies_cpu.clone(),
                    tx_empty_buffers.clone(),
                    tx_nonce_data.clone(),
                )
            });
        }

        for _ in 0..gpu_worker_thread_count {
            thread::spawn({
                create_worker_task(
                    cfg.benchmark_only.to_uppercase() == "I/O",
                    rx_read_replies_gpu.clone(),
                    tx_empty_buffers.clone(),
                    tx_nonce_data.clone(),
                )
            });
        }

        let core = Core::new().unwrap();
        Miner {
            reader_task_count: drive_id_to_plots.len(),
            reader: Reader::new(
                drive_id_to_plots,
                total_size,
                reader_thread_count,
                rx_empty_buffers,
                tx_read_replies_cpu,
                tx_read_replies_gpu,
                cfg.show_progress,
                cfg.show_drive_stats,
                cfg.cpu_thread_pinning,
            ),
            rx_nonce_data,
            target_deadline: cfg.target_deadline,
            request_handler: RequestHandler::new(
                cfg.url,
                cfg.account_id_to_secret_phrase,
                cfg.timeout,
                core.handle(),
                total_size as usize * 4096 / 1024 / 1024 / 1024,
                cfg.send_proxy_details,
            ),
            state: Arc::new(Mutex::new(State {
                height: 0,
                account_id_to_best_deadline: HashMap::new(),
                base_target: 1,
                processed_reader_tasks: 0,
                sw: Stopwatch::new(),
                scanning: false,
            })),
            get_mining_info_interval: cfg.get_mining_info_interval,
            core,
            wakeup_after: cfg.hdd_wakeup_after * 1000, // ms -> s
        }
    }

    pub fn run(mut self) {
        let handle = self.core.handle();
        let request_handler = self.request_handler.clone();
        let total_size = self.reader.total_size;
 
        // you left me no choice!!! at least not one that I could have worked out in two weeks...
        let reader = Rc::new(RefCell::new(self.reader));

        let state = self.state.clone();
        // there might be a way to solve this without two nested moves
        let get_mining_info_interval = self.get_mining_info_interval;
        let wakeup_after = self.wakeup_after;
        handle.spawn(
            Interval::new(
                Instant::now(),
                Duration::from_millis(get_mining_info_interval),
            ).for_each(move |_| {
                let state = state.clone();
                let reader = reader.clone();
                request_handler.get_mining_info().then(move |mining_info| {
                    match mining_info {
                        Ok(mining_info) => {
                            let mut state = state.lock().unwrap();
                            if mining_info.height > state.height {
                                for best_deadlines in state.account_id_to_best_deadline.values_mut()
                                {
                                    *best_deadlines = u64::MAX;
                                }
                                state.height = mining_info.height;
                                state.base_target = mining_info.base_target;

                                let gensig =
                                    burstmath::decode_gensig(&mining_info.generation_signature);
                                let scoop = burstmath::calculate_scoop(mining_info.height, &gensig);
                                info!(
                                    "{: <80}",
                                    format!(
                                        "new block: height={}, scoop={}",
                                        mining_info.height, scoop
                                    )
                                );

                                reader.borrow_mut().start_reading(
                                    mining_info.height,
                                    scoop,
                                    &Arc::new(gensig),
                                );
                                state.sw.restart();
                                state.processed_reader_tasks = 0;
                                state.scanning = true;
                            } else if !state.scanning
                                && wakeup_after != 0
                                && state.sw.elapsed_ms() > wakeup_after
                            {
                                info!("HDD, wakeup!");
                                reader.borrow_mut().wakeup();
                                state.sw.restart();
                            }
                        }
                        _ => warn!("{: <80}", "error getting mining info"),
                    }
                    future::ok(())
                })
            }).map_err(|e| panic!("interval errored: err={:?}", e)),
        );

        let target_deadline = self.target_deadline;
        let request_handler = self.request_handler.clone();
        let inner_handle = handle.clone();
        let state = self.state.clone();
        let reader_task_count = self.reader_task_count;
        handle.spawn(
            self.rx_nonce_data
                .for_each(move |nonce_data| {
                    let mut state = state.lock().unwrap();
                    let deadline = nonce_data.deadline / state.base_target;
                    let best_deadline = *state
                        .account_id_to_best_deadline
                        .get(&nonce_data.account_id)
                        .unwrap_or(&u64::MAX);
                    if best_deadline > deadline && deadline < target_deadline {
                        state
                            .account_id_to_best_deadline
                            .insert(nonce_data.account_id, deadline);
                        request_handler.submit_nonce(
                            &inner_handle,
                            nonce_data.account_id,
                            nonce_data.nonce,
                            nonce_data.height,
                            nonce_data.deadline,
                            deadline,
                            0,
                        );
                        /* tradeoff between non-verbosity and information: stopped informing about
                           found deadlines, but reporting accepted deadlines instead.  
                        info!(
                            "deadline captured: account={}, nonce={}, deadline={}",
                            nonce_data.account_id, nonce_data.nonce, deadline
                        );*/
                    }
                    if nonce_data.reader_task_processed {
                        state.processed_reader_tasks += 1;
                        if state.processed_reader_tasks == reader_task_count {
                            info!(
                                "{: <80}",
                                format!("round finished: roundtime={}ms, speed={:.2}MiB/s", state.sw.elapsed_ms(), total_size as f64 * 1000.0 / 1024.0 / 1024.0 / state.sw.elapsed_ms() as f64)
                            );
                            state.sw.restart();
                            state.scanning = false;
                        }
                    }
                    Ok(())
                }).map_err(|e| panic!("interval errored: err={:?}", e)),
        );

        self.core.run(future::empty::<(), ()>()).unwrap();
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_new_miner() {}
}
