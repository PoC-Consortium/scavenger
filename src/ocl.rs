extern crate aligned_alloc;
extern crate ocl_core as core;
extern crate page_size;

use self::core::{
    ArgVal, ContextProperties, DeviceInfo, Event, KernelWorkGroupInfo, PlatformInfo, Status,
};

use config::Cfg;
use miner::Buffer;
use std::ffi::CString;

use std::process;
use std::sync::{Arc, Mutex};
use std::u64;

static SRC: &'static str = include_str!("ocl/kernel.cl");

// convert the info or error to a string for printing:
macro_rules! to_string {
    ($expr:expr) => {
        match $expr {
            Ok(info) => info.to_string(),
            Err(err) => match err.api_status() {
                Some(Status::CL_KERNEL_ARG_INFO_NOT_AVAILABLE) => "Not available".into(),
                _ => err.to_string(),
            },
        }
    };
}

pub fn platform_info() {
    let platform_ids = core::get_platform_ids().unwrap();
    for (i, platform_id) in platform_ids.iter().enumerate() {
        info!(
            "OCL: platform {}, {} - {}",
            i,
            to_string!(core::get_platform_info(&platform_id, PlatformInfo::Name)),
            to_string!(core::get_platform_info(&platform_id, PlatformInfo::Version))
        );
        let device_ids = core::get_device_ids(&platform_id, None, None).unwrap();
        for (j, device_id) in device_ids.iter().enumerate() {
            info!(
                "OCL: device {}, {} - {}",
                j,
                to_string!(core::get_device_info(device_id, DeviceInfo::Vendor)),
                to_string!(core::get_device_info(device_id, DeviceInfo::Name))
            );
        }
    }
}

pub fn gpu_info(cfg: &Cfg) {
    if cfg.gpu_worker_thread_count > 0 {
        let platform_ids = core::get_platform_ids().unwrap();
        if cfg.gpu_platform >= platform_ids.len() {
            error!("OCL: Selected OpenCL platform doesn't exist. Shutting down...");
            process::exit(0);
        }
        let platform = platform_ids[cfg.gpu_platform];
        let device_ids = core::get_device_ids(&platform, None, None).unwrap();
        if cfg.gpu_device >= device_ids.len() {
            error!("OCL: Selected OpenCL device doesn't exist. Shutting down...");
            process::exit(0);
        }
        let device = device_ids[cfg.gpu_device];
        info!(
            "OCL: {} - {}",
            to_string!(core::get_platform_info(platform, PlatformInfo::Name)),
            to_string!(core::get_platform_info(platform, PlatformInfo::Version))
        );
        info!(
            "GPU: {} - {}",
            to_string!(core::get_device_info(&device, DeviceInfo::Vendor)),
            to_string!(core::get_device_info(&device, DeviceInfo::Name))
        );
        match core::get_device_info(&device, DeviceInfo::GlobalMemSize).unwrap() {
            core::DeviceInfoResult::GlobalMemSize(mem) => {
                info!(
                    "GPU: RAM={}MiB, Cores={}",
                    mem / 1024 / 1024,
                    to_string!(core::get_device_info(&device, DeviceInfo::MaxComputeUnits))
                );
                info!(
                    "GPU: RAM usage (estimated)={}MiB",
                    cfg.gpu_nonces_per_cache * 75 * 2 * cfg.gpu_worker_thread_count / 1024 / 1024
                        + cfg.gpu_worker_thread_count * 45
                );

                // yellow card
                if cfg.gpu_nonces_per_cache * 80 * 2 * cfg.gpu_worker_thread_count / 1024 / 1024
                    > mem as usize / 1024 / 1024
                {
                    warn!(
                        "GPU: Low on GPU memory. If your settings don't work, \
                         please reduce gpu_worker_threads and/or gpu_nonces_per_cache."
                    );
                }

                //red card
                if cfg.gpu_nonces_per_cache * 72 * 2 * cfg.gpu_worker_thread_count / 1024 / 1024
                    > mem as usize / 1024 / 1024
                {
                    error!(
                        "GPU: Insufficient GPU memory. Please reduce gpu_worker_threads \
                         and/or gpu_nonces_per_cache. Shutting down..."
                    );
                    process::exit(0);
                }
            }
            _ => panic!("Unexpected error. Can't obtain GPU memory size."),
        }
    } else if cfg.cpu_worker_thread_count == 0 {
        error!("CPU, GPU: no workers configured. Shutting down...");
        process::exit(0);
    }
}

pub struct GpuContext {
    context: core::Context,
    queue: core::CommandQueue,
    kernel1: core::Kernel,
    kernel2: core::Kernel,
    ldim1: [usize; 3],
    gdim1: [usize; 3],
    ldim2: [usize; 3],
    gdim2: [usize; 3],
    mapping: bool,
}

pub struct GpuBuffer {
    data: Arc<Mutex<Vec<u8>>>,
    context: Arc<Mutex<GpuContext>>,
    gensig_gpu: core::Mem,
    data_gpu: core::Mem,
    deadlines_gpu: core::Mem,
    best_deadline_gpu: core::Mem,
    best_offset_gpu: core::Mem,
    memmap: Option<Arc<core::MemMap<u8>>>,
}

impl GpuBuffer {
    pub fn new(context_mu: &Arc<Mutex<GpuContext>>) -> Self
    where
        Self: Sized,
    {
        let context = context_mu.lock().unwrap();

        let gensig_gpu = unsafe {
            core::create_buffer::<_, u8>(&context.context, core::MEM_READ_ONLY, 32, None).unwrap()
        };

        let deadlines_gpu = unsafe {
            core::create_buffer::<_, u64>(
                &context.context,
                core::MEM_READ_WRITE,
                context.gdim1[0],
                None,
            ).unwrap()
        };

        let best_offset_gpu = unsafe {
            core::create_buffer::<_, u64>(&context.context, core::MEM_READ_WRITE, 1, None).unwrap()
        };

        let best_deadline_gpu = unsafe {
            core::create_buffer::<_, u64>(&context.context, core::MEM_READ_WRITE, 1, None).unwrap()
        };

        let pointer = aligned_alloc::aligned_alloc(&context.gdim1[0] * 64, page_size::get());
        let data: Vec<u8>;
        unsafe {
            data = Vec::from_raw_parts(
                pointer as *mut u8,
                &context.gdim1[0] * 64,
                &context.gdim1[0] * 64,
            );
        }

        let data_gpu = unsafe {
            core::create_buffer(
                &context.context,
                core::MEM_READ_ONLY | core::MEM_USE_HOST_PTR,
                context.gdim1[0] * 64,
                Some(&data),
            ).unwrap()
        };

        GpuBuffer {
            data: Arc::new(Mutex::new(data)),
            context: context_mu.clone(),
            gensig_gpu,
            data_gpu,
            deadlines_gpu,
            best_deadline_gpu,
            best_offset_gpu,
            memmap: None,
        }
    }
}

impl Buffer for GpuBuffer {
    fn get_buffer_for_writing(&mut self) -> Arc<Mutex<Vec<u8>>> {
        // pointer is cached, however, calling enqueue map to make DMA work.
        let locked_context = self.context.lock().unwrap();
        if locked_context.mapping {
            unsafe {
                self.memmap = Some(Arc::new(
                    core::enqueue_map_buffer::<u8, _, _, _>(
                        &(*locked_context).queue,
                        &self.data_gpu,
                        true,
                        core::MAP_WRITE,
                        0,
                        &(*locked_context).gdim1[0] * 64,
                        None::<Event>,
                        None::<&mut Event>,
                    ).unwrap(),
                ));
            }
        }
        self.data.clone()
    }

    fn get_buffer(&mut self) -> Arc<Mutex<Vec<u8>>> {
        self.data.clone()
    }

    fn get_gpu_context(&self) -> Option<Arc<Mutex<GpuContext>>> {
        Some(self.context.clone())
    }
    fn get_gpu_buffers(&self) -> Option<&GpuBuffer> {
        Some(self)
    }
}

// Ohne Gummi im Bahnhofsviertel... das wird noch Konsequenzen haben
unsafe impl Sync for GpuContext {}
unsafe impl Send for GpuBuffer {}

impl GpuContext {
    pub fn new(
        gpu_platform: usize,
        gpu_id: usize,
        nonces_per_cache: usize,
        mapping: bool,
    ) -> GpuContext {
        let platform_ids = core::get_platform_ids().unwrap();
        let platform_id = platform_ids[gpu_platform];
        let device_ids = core::get_device_ids(&platform_id, None, None).unwrap();
        let device_id = device_ids[gpu_id];
        let context_properties = ContextProperties::new().platform(platform_id);
        let context =
            core::create_context(Some(&context_properties), &[device_id], None, None).unwrap();
        let src_cstring = CString::new(SRC).unwrap();
        let program = core::create_program_with_source(&context, &[src_cstring]).unwrap();
        core::build_program(
            &program,
            None::<&[()]>,
            &CString::new("").unwrap(),
            None,
            None,
        ).unwrap();
        let queue = core::create_command_queue(&context, &device_id, None).unwrap();
        let kernel1 = core::create_kernel(&program, "calculate_deadlines").unwrap();
        let kernel2 = core::create_kernel(&program, "find_min").unwrap();

        let kernel1_workgroup_size = get_kernel_work_group_size(&kernel1, device_id);
        let kernel2_workgroup_size = get_kernel_work_group_size(&kernel2, device_id);

        let mut workgroup_count = nonces_per_cache / kernel1_workgroup_size;
        if nonces_per_cache % kernel1_workgroup_size != 0 {
            workgroup_count += 1;
        }

        let gdim1 = [kernel1_workgroup_size * workgroup_count, 1, 1];
        let ldim1 = [kernel1_workgroup_size, 1, 1];
        let gdim2 = [kernel2_workgroup_size, 1, 1];
        let ldim2 = [kernel2_workgroup_size, 1, 1];

        GpuContext {
            context,
            queue,
            kernel1,
            kernel2,
            ldim1,
            gdim1,
            ldim2,
            gdim2,
            mapping,
        }
    }
}

pub fn find_best_deadline_gpu(
    buffer: &GpuBuffer,
    nonce_count: usize,
    gensig: [u8; 32],
) -> (u64, u64) {
    let data = buffer.data.clone();
    let data2 = (*data).lock().unwrap();
    let gpu_context_mtx = (*buffer).get_gpu_context().unwrap();
    let gpu_context = gpu_context_mtx.lock().unwrap();

    unsafe {
        core::enqueue_write_buffer(
            &gpu_context.queue,
            &buffer.gensig_gpu,
            false,
            0,
            &gensig,
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }

    if gpu_context.mapping {
        let temp = buffer.memmap.clone();
        let temp2 = temp.unwrap();
        core::enqueue_unmap_mem_object(
            &gpu_context.queue,
            &buffer.data_gpu,
            &*temp2,
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    } else {
        unsafe {
            core::enqueue_write_buffer(
                &gpu_context.queue,
                &buffer.data_gpu,
                false,
                0,
                &data2,
                None::<Event>,
                None::<&mut Event>,
            ).unwrap();
        }
    }

    core::set_kernel_arg(&gpu_context.kernel1, 0, ArgVal::mem(&buffer.gensig_gpu)).unwrap();
    core::set_kernel_arg(&gpu_context.kernel1, 1, ArgVal::mem(&buffer.data_gpu)).unwrap();
    core::set_kernel_arg(&gpu_context.kernel1, 2, ArgVal::mem(&buffer.deadlines_gpu)).unwrap();

    unsafe {
        core::enqueue_kernel(
            &gpu_context.queue,
            &gpu_context.kernel1,
            1,
            None,
            &gpu_context.gdim1,
            Some(gpu_context.ldim1),
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }

    core::set_kernel_arg(&gpu_context.kernel2, 0, ArgVal::mem(&buffer.deadlines_gpu)).unwrap();
    core::set_kernel_arg(&gpu_context.kernel2, 1, ArgVal::primitive(&(nonce_count as u64))).unwrap();
    core::set_kernel_arg(
        &gpu_context.kernel2,
        2,
        ArgVal::local::<u32>(&gpu_context.ldim2[0]),
    ).unwrap();
    core::set_kernel_arg(
        &gpu_context.kernel2,
        3,
        ArgVal::mem(&buffer.best_offset_gpu),
    ).unwrap();
    core::set_kernel_arg(
        &gpu_context.kernel2,
        4,
        ArgVal::mem(&buffer.best_deadline_gpu),
    ).unwrap();

    unsafe {
        core::enqueue_kernel(
            &gpu_context.queue,
            &gpu_context.kernel2,
            1,
            None,
            &gpu_context.gdim2,
            Some(gpu_context.ldim2),
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }

    let mut best_offset = vec![0u64; 1];
    let mut best_deadline = vec![0u64; 1];

    unsafe {
        core::enqueue_read_buffer(
            &gpu_context.queue,
            &buffer.best_offset_gpu,
            true,
            0,
            &mut best_offset,
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }
    unsafe {
        core::enqueue_read_buffer(
            &gpu_context.queue,
            &buffer.best_deadline_gpu,
            true,
            0,
            &mut best_deadline,
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }

    (best_deadline[0], best_offset[0])
}

fn get_kernel_work_group_size(x: &core::Kernel, y: core::DeviceId) -> usize {
    match core::get_kernel_work_group_info(x, y, KernelWorkGroupInfo::WorkGroupSize).unwrap() {
        core::KernelWorkGroupInfoResult::WorkGroupSize(kws) => kws,
        _ => panic!("Unexpected error"),
    }
}
