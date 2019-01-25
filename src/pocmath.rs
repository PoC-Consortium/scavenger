use hex;
use shabals;
use std::mem::transmute;

pub fn decode_gensig(gensig: &str) -> [u8; 32] {
    let mut gensig_bytes = [0; 32];
    gensig_bytes[..].clone_from_slice(&hex::decode(gensig).unwrap());
    gensig_bytes
}

pub fn calculate_scoop(height: u64, gensig: &[u8; 32]) -> u32 {
    let mut data: [u8; 40] = [0; 40];
    let height_bytes: [u8; 8] = unsafe { transmute(height.to_be()) };

    data[32..].clone_from_slice(&height_bytes);
    data[..32].clone_from_slice(gensig);

    let new_gensig = shabals::shabal256(&data);
    (u32::from(new_gensig[30] & 0x0F) << 8) | u32::from(new_gensig[31])
}
