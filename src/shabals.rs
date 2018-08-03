#![allow(dead_code)]

use libc::{c_uchar, c_uint, c_void, size_t, uint32_t};

use std::mem::zeroed;

#[repr(C)]
#[allow(non_snake_case)]
pub struct ShabalContext {
    pub buf: [c_uchar; 64usize],
    pub ptr: size_t,
    pub state: [uint32_t; 12 + 16 + 16],
    pub Wlow: uint32_t,
    pub Whigh: uint32_t,
    pub out_size: uint32_t,
}

impl ::std::default::Default for ShabalContext {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

pub fn to_void_raw_ctx<T>(cc: &mut T) -> *mut c_void {
    let raw_cc = cc as *mut T;
    let void_raw_cc = raw_cc as *mut c_void;

    return void_raw_cc;
}

pub fn to_void_raw_data(data: &[u8]) -> (*const c_void, size_t) {
    let void_raw_data = data.as_ptr() as *const c_void;
    let len = data.len() as size_t;

    return (void_raw_data, len);
}

pub fn to_void_raw_dest(dest: &mut [u8]) -> *mut c_void {
    let raw_dest = dest.as_mut() as *mut [u8];
    let void_raw_dest = raw_dest as *mut c_void;

    return void_raw_dest;
}

extern "C" {
    pub fn shabal_init(cc: *mut c_void, out_size: c_uint) -> ();
    pub fn shabal(cc: *mut c_void, data: *const c_void, len: size_t) -> ();
    pub fn shabal_close(cc: *mut c_void, ub: c_uint, n: c_uint, dst: *mut c_void) -> ();
}

pub fn shabal256_init(cc: &mut ShabalContext) {
    let void_raw_cc = to_void_raw_ctx(cc);
    unsafe { shabal_init(void_raw_cc, 256) };
}

pub fn shabal256_load(cc: &mut ShabalContext, data: &[u8]) {
    let void_raw_cc = to_void_raw_ctx(cc);
    let (void_raw_data, len) = to_void_raw_data(data);
    unsafe { shabal(void_raw_cc, void_raw_data, len) };
}

pub fn shabal256_close(cc: &mut ShabalContext, dest: &mut [u8; 32]) {
    let void_raw_cc = to_void_raw_ctx(cc);
    let void_raw_dest = to_void_raw_dest(dest);
    unsafe {
        shabal_close(void_raw_cc, 0, 0, void_raw_dest);
    };
}

pub fn shabal256(data: &[u8]) -> [u8; 32] {
    let mut dest = [0; 32];
    let mut cc = ShabalContext::default();

    shabal256_init(&mut cc);
    shabal256_load(&mut cc, data);
    shabal256_close(&mut cc, &mut dest);

    return dest;
}
