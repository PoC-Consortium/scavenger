//! Get information about all the things using `core` function calls.
//!
//! Set `INFO_FORMAT_MULTILINE` to `false` for compact printing.

extern crate ocl_core as core;
use libc::c_void;
//use self::core::{ArgVal, ContextProperties, DeviceInfo, Event, PlatformInfo, Status};
use self::core::{
    ArgVal, ContextProperties, DeviceInfo, Event, KernelWorkGroupInfo, PlatformInfo, Status,
};

use config::Cfg;
use std::ffi::CString;
use std::mem;
use std::u64;

static SRC: &'static str = include_str!("ocl/kernel.cl");

/// Convert the info or error to a string for printing:
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

pub fn init_gpu(cfg: &Cfg) {
    // display info
    let platform_ids = core::get_platform_ids().unwrap();
    let platform = platform_ids[cfg.gpu_platform];
    let device_ids = core::get_device_ids(&platform, None, None).unwrap();
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
                "GPU: RAM={}MB, Cores={}",
                mem / 1024 / 1024,
                to_string!(core::get_device_info(&device, DeviceInfo::MaxComputeUnits))
            );
        }
        _ => panic!("Unexpected error"),
    }
}

pub fn find_best_deadline_gpu(
    scoops: *const c_void,
    nonce_count: usize,
    gensig: [u8; 32],
) -> (u64, u64) {
    //WORK IN PROGRESS!!!

    // (1) Define which platform and device(s) to use. Create a context,
    // queue, and program

    let platform_id = core::default_platform().unwrap();
    let device_ids = core::get_device_ids(&platform_id, None, None).unwrap();
    let device_id = device_ids[0];
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

    //get kernel workgroup sizes
    let kernel1_workgroup_size = get_kernel_work_group_size(&kernel1, &device_id);
    let kernel2_workgroup_size = get_kernel_work_group_size(&kernel2, &device_id);

    //End init for now... TODO amend to fixed cache via cfg

    let mut workgroup_count = nonce_count / kernel1_workgroup_size;
    if nonce_count % kernel1_workgroup_size != 0 {
        workgroup_count = workgroup_count + 1;
    }

    //Define Dimensions
    let gdim1 = [kernel1_workgroup_size * workgroup_count, 1, 1];
    let ldim1 = [kernel1_workgroup_size, 1, 1];
    let gdim2 = [kernel2_workgroup_size, 1, 1];
    let ldim2 = [kernel2_workgroup_size, 1, 1];

    // Prepare kernel 1
    let gensig_gpu =
        unsafe { core::create_buffer::<_, u8>(&context, core::MEM_READ_ONLY, 32, None).unwrap() };

    //scoops
    let data_gpu = unsafe {
        core::create_buffer::<_, u8>(&context, core::MEM_READ_ONLY, gdim1[0] * 64, None).unwrap()
    };

    //deadlines, 64bit (8 byte) per deadline, read & write
    let deadlines_gpu = unsafe {
        core::create_buffer::<_, u64>(&context, core::MEM_READ_WRITE, gdim1[0], None).unwrap()
    };

    let mut event1 = Event::null();
    let mut event2 = Event::null();

    //real init end

    //cast nonce data
    let data: Vec<u8>;
    unsafe {
        data = Vec::from_raw_parts(scoops as *mut u8, nonce_count * 64, nonce_count * 64);
    }

    unsafe {
        core::enqueue_write_buffer(
            &queue,
            &gensig_gpu,
            false,
            0,
            &gensig,
            None::<Event>,
            Some(&mut event1),
        ).unwrap();
    }

    unsafe {
        core::enqueue_write_buffer(
            &queue,
            &data_gpu,
            false,
            0,
            &data,
            None::<Event>,
            Some(&mut event2),
        ).unwrap();
    }

    core::set_kernel_arg(&kernel1, 0, ArgVal::mem(&gensig_gpu)).unwrap();
    core::set_kernel_arg(&kernel1, 1, ArgVal::mem(&data_gpu)).unwrap();
    core::set_kernel_arg(&kernel1, 2, ArgVal::mem(&deadlines_gpu)).unwrap();

    // Run kernel1: Calculate deadlines
    unsafe {
        // (4) Run the kernel:
        core::enqueue_kernel(
            &queue,
            &kernel1,
            1,
            None,
            &gdim1,
            Some(ldim1),
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }

    // Prepare kernel 2
    let mut best_offset = vec![0u64; 1];
    let best_offset_gpu = unsafe {
        core::create_buffer(&context, core::MEM_READ_WRITE, 1, Some(&best_offset)).unwrap()
    };

    let mut best_deadline = vec![0u64; 1];
    let best_deadline_gpu = unsafe {
        core::create_buffer(&context, core::MEM_READ_WRITE, 1, Some(&best_deadline)).unwrap()
    };

    core::set_kernel_arg(&kernel2, 0, ArgVal::mem(&deadlines_gpu)).unwrap();
    core::set_kernel_arg(&kernel2, 1, ArgVal::primitive(&nonce_count)).unwrap();
    core::set_kernel_arg(&kernel2, 2, ArgVal::local::<u32>(&ldim2[0])).unwrap();
    core::set_kernel_arg(&kernel2, 3, ArgVal::mem(&best_offset_gpu)).unwrap();
    core::set_kernel_arg(&kernel2, 4, ArgVal::mem(&best_deadline_gpu)).unwrap();

    // Run kernel2: Calculate deadlines
    unsafe {
        // (4) Run the kernel:
        core::enqueue_kernel(
            &queue,
            &kernel2,
            1,
            None,
            &gdim2,
            Some(ldim2),
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }

    //download
    unsafe {
        core::enqueue_read_buffer(
            &queue,
            &best_offset_gpu,
            true,
            0,
            &mut best_offset,
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }
    unsafe {
        core::enqueue_read_buffer(
            &queue,
            &best_deadline_gpu,
            true,
            0,
            &mut best_deadline,
            None::<Event>,
            None::<&mut Event>,
        ).unwrap();
    }

    //Die Zeit heilt Wunden doch vergessen kann ich nicht...
    mem::forget(data);
    (best_deadline[0], best_offset[0])
}

fn get_kernel_work_group_size(x: &core::Kernel, y: &core::DeviceId) -> usize {
    match core::get_kernel_work_group_info(x, y, KernelWorkGroupInfo::WorkGroupSize).unwrap() {
        core::KernelWorkGroupInfoResult::WorkGroupSize(kws) => kws,
        _ => panic!("Unexpected error"),
    }
}
