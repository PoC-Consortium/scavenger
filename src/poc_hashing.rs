use crate::shabal256::{shabal256_deadline_fast, shabal256_hash_fast};
use hex;
use std::mem::transmute;
use std::u64;

const SCOOP_SIZE: usize = 64;

pub fn decode_gensig(gensig: &str) -> [u8; 32] {
    let mut gensig_bytes = [0; 32];
    gensig_bytes[..].clone_from_slice(&hex::decode(gensig).unwrap());
    gensig_bytes
}

pub fn calculate_scoop(height: u64, gensig: &[u8; 32]) -> u32 {
    let mut data: [u8; 64] = [0; 64];
    let height_bytes: [u8; 8] = unsafe { transmute(height.to_be()) };

    data[..32].clone_from_slice(gensig);
    data[32..40].clone_from_slice(&height_bytes);
    data[40] = 0x80;
    let data = unsafe { std::mem::transmute::<&[u8; 64], &[u32; 16]>(&data) };

    let new_gensig = &shabal256_hash_fast(&[], &data);
    (u32::from(new_gensig[30] & 0x0F) << 8) | u32::from(new_gensig[31])
}

pub fn find_best_deadline_rust(
    data: &[u8],
    number_of_nonces: u64,
    gensig: &[u8; 32],
) -> (u64, u64) {
    let mut best_deadline = u64::MAX;
    let mut best_offset = 0;
    for i in 0..number_of_nonces as usize {
        let result =
            shabal256_deadline_fast(&data[i * SCOOP_SIZE..i * SCOOP_SIZE + SCOOP_SIZE], &gensig);
        if result < best_deadline {
            best_deadline = result;
            best_offset = i;
        }
    }
    (best_deadline, best_offset as u64)
}
