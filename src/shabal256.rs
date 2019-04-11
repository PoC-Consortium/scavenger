use std::slice::from_raw_parts;

const A_INIT: [u32; 12] = [
    0x52F84552, 0xE54B7999, 0x2D8EE3EC, 0xB9645191, 0xE0078B86, 0xBB7C44C9, 0xD2B5C1CA, 0xB0D2EB8C,
    0x14CE5A45, 0x22AF50DC, 0xEFFDBC6B, 0xEB21B74A,
];

const B_INIT: [u32; 16] = [
    0xB555C6EE, 0x3E710596, 0xA72A652F, 0x9301515F, 0xDA28C1FA, 0x696FD868, 0x9CB6BF72, 0x0AFE4002,
    0xA6E03615, 0x5138C1D4, 0xBE216306, 0xB38B8890, 0x3EA8B96B, 0x3299ACE4, 0x30924DD4, 0x55CB34A5,
];

const C_INIT: [u32; 16] = [
    0xB405F031, 0xC4233EBA, 0xB3733979, 0xC0DD9D55, 0xC51C28AE, 0xA327B8E1, 0x56C56167, 0xED614433,
    0x88B59D60, 0x60E2CEBA, 0x758B4B8B, 0x83E82A7F, 0xBC968828, 0xE6E00BF7, 0xBA839E55, 0x9B491C60,
];

pub fn shabal256_deadline_fast(data: &[u8], gensig: &[u8; 32]) -> u64 {
    let mut a = A_INIT;
    let mut b = B_INIT;
    let mut c = C_INIT;
    let mut w_high = 0u32;
    let mut w_low = 1u32;
    let data_ptr = data.as_ptr() as *const u32;
    let data = unsafe { from_raw_parts(data_ptr, data.len() / 4) };
    let gensig = unsafe { std::mem::transmute::<&[u8; 32], &[u32; 8]>(&gensig) };
    let mut term = [0u32; 8];
    term[0] = 0x80;

    input_block_add_dl(&mut b, &gensig[..], &data[..8]);
    xor_w(&mut a, w_low, w_high);
    apply_p_dl(&mut a, &mut b, &c, gensig, &data[..8]);
    input_block_sub_dl(&mut c, gensig, &data[..8]);
    swap_bc(&mut b, &mut c);
    incr_w(&mut w_low, &mut w_high);

    input_block_add_dl(&mut b, &data[8..], &term);
    xor_w(&mut a, w_low, w_high);
    apply_p_dl(&mut a, &mut b, &c, &data[8..], &term);
    for _ in 0..3 {
        swap_bc(&mut b, &mut c);
        xor_w(&mut a, w_low, w_high);
        apply_p_dl(&mut a, &mut b, &c, &data[8..], &term);
    }
    let b = unsafe { std::mem::transmute::<&[u32; 16], &[u64; 8]>(&b) };
    b[4]
}

pub fn shabal256_hash_fast(data: &[u8], term: &[u32; 16]) -> [u8; 32] {
    let mut a = A_INIT;
    let mut b = B_INIT;
    let mut c = C_INIT;
    let mut w_high = 0u32;
    let mut w_low = 1u32;
    let mut num = data.len() >> 6;
    let mut ptr = 0;
    let data_ptr = data.as_ptr() as *const u32;
    let data = unsafe { from_raw_parts(data_ptr, data.len() / 4) };

    while num > 0 {
        input_block_add(&mut b, &data[ptr..]);
        xor_w(&mut a, w_low, w_high);
        apply_p(&mut a, &mut b, &c, &data[ptr..]);
        input_block_sub(&mut c, &data[ptr..]);
        swap_bc(&mut b, &mut c);
        incr_w(&mut w_low, &mut w_high);
        ptr = ptr.wrapping_add(16);
        num = num.wrapping_sub(1);
    }
    input_block_add(&mut b, term);
    xor_w(&mut a, w_low, w_high);
    apply_p(&mut a, &mut b, &c, term);
    for _ in 0..3 {
        swap_bc(&mut b, &mut c);
        xor_w(&mut a, w_low, w_high);
        apply_p(&mut a, &mut b, &c, term);
    }
    unsafe { *(b[8..16].as_ptr() as *const [u8; 32]) }
}

#[inline(always)]
fn input_block_add(b: &mut [u32; 16], data: &[u32]) {
    for (element, data) in b.iter_mut().zip(data.iter()) {
        *element = element.wrapping_add(*data);
    }
}

#[inline(always)]
fn input_block_add_dl(b: &mut [u32; 16], data_a: &[u32], data_b: &[u32]) {
    unsafe {
        *b.get_unchecked_mut(0) = b
            .get_unchecked_mut(0)
            .wrapping_add(*data_a.get_unchecked(0));
        *b.get_unchecked_mut(1) = b
            .get_unchecked_mut(1)
            .wrapping_add(*data_a.get_unchecked(1));
        *b.get_unchecked_mut(2) = b
            .get_unchecked_mut(2)
            .wrapping_add(*data_a.get_unchecked(2));
        *b.get_unchecked_mut(3) = b
            .get_unchecked_mut(3)
            .wrapping_add(*data_a.get_unchecked(3));
        *b.get_unchecked_mut(4) = b
            .get_unchecked_mut(4)
            .wrapping_add(*data_a.get_unchecked(4));
        *b.get_unchecked_mut(5) = b
            .get_unchecked_mut(5)
            .wrapping_add(*data_a.get_unchecked(5));
        *b.get_unchecked_mut(6) = b
            .get_unchecked_mut(6)
            .wrapping_add(*data_a.get_unchecked(6));
        *b.get_unchecked_mut(7) = b
            .get_unchecked_mut(7)
            .wrapping_add(*data_a.get_unchecked(7));
        *b.get_unchecked_mut(8) = b
            .get_unchecked_mut(8)
            .wrapping_add(*data_b.get_unchecked(0));
        *b.get_unchecked_mut(9) = b
            .get_unchecked_mut(9)
            .wrapping_add(*data_b.get_unchecked(1));
        *b.get_unchecked_mut(10) = b
            .get_unchecked_mut(10)
            .wrapping_add(*data_b.get_unchecked(2));
        *b.get_unchecked_mut(11) = b
            .get_unchecked_mut(11)
            .wrapping_add(*data_b.get_unchecked(3));
        *b.get_unchecked_mut(12) = b
            .get_unchecked_mut(12)
            .wrapping_add(*data_b.get_unchecked(4));
        *b.get_unchecked_mut(13) = b
            .get_unchecked_mut(13)
            .wrapping_add(*data_b.get_unchecked(5));
        *b.get_unchecked_mut(14) = b
            .get_unchecked_mut(14)
            .wrapping_add(*data_b.get_unchecked(6));
        *b.get_unchecked_mut(15) = b
            .get_unchecked_mut(15)
            .wrapping_add(*data_b.get_unchecked(7));
    }
}

#[inline(always)]
fn input_block_sub(c: &mut [u32; 16], data: &[u32]) {
    for (element, data) in c.iter_mut().zip(data.iter()) {
        *element = element.wrapping_sub(*data);
    }
}

#[inline(always)]
fn input_block_sub_dl(b: &mut [u32; 16], data_a: &[u32], data_b: &[u32]) {
    unsafe {
        *b.get_unchecked_mut(0) = b
            .get_unchecked_mut(0)
            .wrapping_sub(*data_a.get_unchecked(0));
        *b.get_unchecked_mut(1) = b
            .get_unchecked_mut(1)
            .wrapping_sub(*data_a.get_unchecked(1));
        *b.get_unchecked_mut(2) = b
            .get_unchecked_mut(2)
            .wrapping_sub(*data_a.get_unchecked(2));
        *b.get_unchecked_mut(3) = b
            .get_unchecked_mut(3)
            .wrapping_sub(*data_a.get_unchecked(3));
        *b.get_unchecked_mut(4) = b
            .get_unchecked_mut(4)
            .wrapping_sub(*data_a.get_unchecked(4));
        *b.get_unchecked_mut(5) = b
            .get_unchecked_mut(5)
            .wrapping_sub(*data_a.get_unchecked(5));
        *b.get_unchecked_mut(6) = b
            .get_unchecked_mut(6)
            .wrapping_sub(*data_a.get_unchecked(6));
        *b.get_unchecked_mut(7) = b
            .get_unchecked_mut(7)
            .wrapping_sub(*data_a.get_unchecked(7));
        *b.get_unchecked_mut(8) = b
            .get_unchecked_mut(8)
            .wrapping_sub(*data_b.get_unchecked(0));
        *b.get_unchecked_mut(9) = b
            .get_unchecked_mut(9)
            .wrapping_sub(*data_b.get_unchecked(1));
        *b.get_unchecked_mut(10) = b
            .get_unchecked_mut(10)
            .wrapping_sub(*data_b.get_unchecked(2));
        *b.get_unchecked_mut(11) = b
            .get_unchecked_mut(11)
            .wrapping_sub(*data_b.get_unchecked(3));
        *b.get_unchecked_mut(12) = b
            .get_unchecked_mut(12)
            .wrapping_sub(*data_b.get_unchecked(4));
        *b.get_unchecked_mut(13) = b
            .get_unchecked_mut(13)
            .wrapping_sub(*data_b.get_unchecked(5));
        *b.get_unchecked_mut(14) = b
            .get_unchecked_mut(14)
            .wrapping_sub(*data_b.get_unchecked(6));
        *b.get_unchecked_mut(15) = b
            .get_unchecked_mut(15)
            .wrapping_sub(*data_b.get_unchecked(7));
    }
}

#[inline(always)]
fn xor_w(a: &mut [u32; 12], w_low: u32, w_high: u32) {
    a[0] ^= w_low;
    a[1] ^= w_high;
}

#[inline(always)]
fn apply_p(a: &mut [u32; 12], b: &mut [u32; 16], c: &[u32; 16], data: &[u32]) {
    for element in b.iter_mut() {
        *element = element.wrapping_shl(17) | element.wrapping_shr(15);
    }
    perm(a, b, c, data);
    a[0] = a[0]
        .wrapping_add(c[11])
        .wrapping_add(c[15])
        .wrapping_add(c[3]);
    a[1] = a[1]
        .wrapping_add(c[12])
        .wrapping_add(c[0])
        .wrapping_add(c[4]);
    a[2] = a[2]
        .wrapping_add(c[13])
        .wrapping_add(c[1])
        .wrapping_add(c[5]);
    a[3] = a[3]
        .wrapping_add(c[14])
        .wrapping_add(c[2])
        .wrapping_add(c[6]);
    a[4] = a[4]
        .wrapping_add(c[15])
        .wrapping_add(c[3])
        .wrapping_add(c[7]);
    a[5] = a[5]
        .wrapping_add(c[0])
        .wrapping_add(c[4])
        .wrapping_add(c[8]);
    a[6] = a[6]
        .wrapping_add(c[1])
        .wrapping_add(c[5])
        .wrapping_add(c[9]);
    a[7] = a[7]
        .wrapping_add(c[2])
        .wrapping_add(c[6])
        .wrapping_add(c[10]);
    a[8] = a[8]
        .wrapping_add(c[3])
        .wrapping_add(c[7])
        .wrapping_add(c[11]);
    a[9] = a[9]
        .wrapping_add(c[4])
        .wrapping_add(c[8])
        .wrapping_add(c[12]);
    a[10] = a[10]
        .wrapping_add(c[5])
        .wrapping_add(c[9])
        .wrapping_add(c[13]);
    a[11] = a[11]
        .wrapping_add(c[6])
        .wrapping_add(c[10])
        .wrapping_add(c[14]);
}

#[inline(always)]
fn apply_p_dl(a: &mut [u32; 12], b: &mut [u32; 16], c: &[u32; 16], data_a: &[u32], data_b: &[u32]) {
    for element in b.iter_mut() {
        *element = element.wrapping_shl(17) | element.wrapping_shr(15);
    }
    perm_dl(a, b, c, data_a, data_b);
    a[0] = a[0]
        .wrapping_add(c[11])
        .wrapping_add(c[15])
        .wrapping_add(c[3]);
    a[1] = a[1]
        .wrapping_add(c[12])
        .wrapping_add(c[0])
        .wrapping_add(c[4]);
    a[2] = a[2]
        .wrapping_add(c[13])
        .wrapping_add(c[1])
        .wrapping_add(c[5]);
    a[3] = a[3]
        .wrapping_add(c[14])
        .wrapping_add(c[2])
        .wrapping_add(c[6]);
    a[4] = a[4]
        .wrapping_add(c[15])
        .wrapping_add(c[3])
        .wrapping_add(c[7]);
    a[5] = a[5]
        .wrapping_add(c[0])
        .wrapping_add(c[4])
        .wrapping_add(c[8]);
    a[6] = a[6]
        .wrapping_add(c[1])
        .wrapping_add(c[5])
        .wrapping_add(c[9]);
    a[7] = a[7]
        .wrapping_add(c[2])
        .wrapping_add(c[6])
        .wrapping_add(c[10]);
    a[8] = a[8]
        .wrapping_add(c[3])
        .wrapping_add(c[7])
        .wrapping_add(c[11]);
    a[9] = a[9]
        .wrapping_add(c[4])
        .wrapping_add(c[8])
        .wrapping_add(c[12]);
    a[10] = a[10]
        .wrapping_add(c[5])
        .wrapping_add(c[9])
        .wrapping_add(c[13]);
    a[11] = a[11]
        .wrapping_add(c[6])
        .wrapping_add(c[10])
        .wrapping_add(c[14]);
}

#[inline(always)]
fn perm_elt(
    a: &mut [u32; 12],
    b: &mut [u32; 16],
    xa0: usize,
    xa1: usize,
    xb0: usize,
    xb1: usize,
    xb2: usize,
    xb3: usize,
    xc: u32,
    xm: u32,
) {
    unsafe {
        *a.get_unchecked_mut(xa0) = (a.get_unchecked(xa0)
            ^ ((a.get_unchecked(xa1).wrapping_shl(15u32)
                | a.get_unchecked(xa1).wrapping_shr(17u32))
            .wrapping_mul(5u32))
            ^ xc)
            .wrapping_mul(3u32)
            ^ b.get_unchecked(xb1)
            ^ (b.get_unchecked(xb2) & !b.get_unchecked(xb3))
            ^ xm;
        *b.get_unchecked_mut(xb0) = !((b.get_unchecked(xb0).wrapping_shl(1)
            | b.get_unchecked(xb0).wrapping_shr(31))
            ^ a.get_unchecked(xa0));
    }
}

#[inline(always)]
fn perm(a: &mut [u32; 12], b: &mut [u32; 16], c: &[u32; 16], data: &[u32]) {
    unsafe {
        perm_elt(a, b, 0, 11, 0, 13, 9, 6, c[8], *data.get_unchecked(0));
        perm_elt(a, b, 1, 0, 1, 14, 10, 7, c[7], *data.get_unchecked(1));
        perm_elt(a, b, 2, 1, 2, 15, 11, 8, c[6], *data.get_unchecked(2));
        perm_elt(a, b, 3, 2, 3, 0, 12, 9, c[5], *data.get_unchecked(3));
        perm_elt(a, b, 4, 3, 4, 1, 13, 10, c[4], *data.get_unchecked(4));
        perm_elt(a, b, 5, 4, 5, 2, 14, 11, c[3], *data.get_unchecked(5));
        perm_elt(a, b, 6, 5, 6, 3, 15, 12, c[2], *data.get_unchecked(6));
        perm_elt(a, b, 7, 6, 7, 4, 0, 13, c[1], *data.get_unchecked(7));
        perm_elt(a, b, 8, 7, 8, 5, 1, 14, c[0], *data.get_unchecked(8));
        perm_elt(a, b, 9, 8, 9, 6, 2, 15, c[15], *data.get_unchecked(9));
        perm_elt(a, b, 10, 9, 10, 7, 3, 0, c[14], *data.get_unchecked(10));
        perm_elt(a, b, 11, 10, 11, 8, 4, 1, c[13], *data.get_unchecked(11));
        perm_elt(a, b, 0, 11, 12, 9, 5, 2, c[12], *data.get_unchecked(12));
        perm_elt(a, b, 1, 0, 13, 10, 6, 3, c[11], *data.get_unchecked(13));
        perm_elt(a, b, 2, 1, 14, 11, 7, 4, c[10], *data.get_unchecked(14));
        perm_elt(a, b, 3, 2, 15, 12, 8, 5, c[9], *data.get_unchecked(15));
        perm_elt(a, b, 4, 3, 0, 13, 9, 6, c[8], *data.get_unchecked(0));
        perm_elt(a, b, 5, 4, 1, 14, 10, 7, c[7], *data.get_unchecked(1));
        perm_elt(a, b, 6, 5, 2, 15, 11, 8, c[6], *data.get_unchecked(2));
        perm_elt(a, b, 7, 6, 3, 0, 12, 9, c[5], *data.get_unchecked(3));
        perm_elt(a, b, 8, 7, 4, 1, 13, 10, c[4], *data.get_unchecked(4));
        perm_elt(a, b, 9, 8, 5, 2, 14, 11, c[3], *data.get_unchecked(5));
        perm_elt(a, b, 10, 9, 6, 3, 15, 12, c[2], *data.get_unchecked(6));
        perm_elt(a, b, 11, 10, 7, 4, 0, 13, c[1], *data.get_unchecked(7));
        perm_elt(a, b, 0, 11, 8, 5, 1, 14, c[0], *data.get_unchecked(8));
        perm_elt(a, b, 1, 0, 9, 6, 2, 15, c[15], *data.get_unchecked(9));
        perm_elt(a, b, 2, 1, 10, 7, 3, 0, c[14], *data.get_unchecked(10));
        perm_elt(a, b, 3, 2, 11, 8, 4, 1, c[13], *data.get_unchecked(11));
        perm_elt(a, b, 4, 3, 12, 9, 5, 2, c[12], *data.get_unchecked(12));
        perm_elt(a, b, 5, 4, 13, 10, 6, 3, c[11], *data.get_unchecked(13));
        perm_elt(a, b, 6, 5, 14, 11, 7, 4, c[10], *data.get_unchecked(14));
        perm_elt(a, b, 7, 6, 15, 12, 8, 5, c[9], *data.get_unchecked(15));
        perm_elt(a, b, 8, 7, 0, 13, 9, 6, c[8], *data.get_unchecked(0));
        perm_elt(a, b, 9, 8, 1, 14, 10, 7, c[7], *data.get_unchecked(1));
        perm_elt(a, b, 10, 9, 2, 15, 11, 8, c[6], *data.get_unchecked(2));
        perm_elt(a, b, 11, 10, 3, 0, 12, 9, c[5], *data.get_unchecked(3));
        perm_elt(a, b, 0, 11, 4, 1, 13, 10, c[4], *data.get_unchecked(4));
        perm_elt(a, b, 1, 0, 5, 2, 14, 11, c[3], *data.get_unchecked(5));
        perm_elt(a, b, 2, 1, 6, 3, 15, 12, c[2], *data.get_unchecked(6));
        perm_elt(a, b, 3, 2, 7, 4, 0, 13, c[1], *data.get_unchecked(7));
        perm_elt(a, b, 4, 3, 8, 5, 1, 14, c[0], *data.get_unchecked(8));
        perm_elt(a, b, 5, 4, 9, 6, 2, 15, c[15], *data.get_unchecked(9));
        perm_elt(a, b, 6, 5, 10, 7, 3, 0, c[14], *data.get_unchecked(10));
        perm_elt(a, b, 7, 6, 11, 8, 4, 1, c[13], *data.get_unchecked(11));
        perm_elt(a, b, 8, 7, 12, 9, 5, 2, c[12], *data.get_unchecked(12));
        perm_elt(a, b, 9, 8, 13, 10, 6, 3, c[11], *data.get_unchecked(13));
        perm_elt(a, b, 10, 9, 14, 11, 7, 4, c[10], *data.get_unchecked(14));
        perm_elt(a, b, 11, 10, 15, 12, 8, 5, c[9], *data.get_unchecked(15));
    }
}

#[inline(always)]
fn perm_dl(a: &mut [u32; 12], b: &mut [u32; 16], c: &[u32; 16], data_a: &[u32], data_b: &[u32]) {
    unsafe {
        perm_elt(a, b, 0, 11, 0, 13, 9, 6, c[8], *data_a.get_unchecked(0));
        perm_elt(a, b, 1, 0, 1, 14, 10, 7, c[7], *data_a.get_unchecked(1));
        perm_elt(a, b, 2, 1, 2, 15, 11, 8, c[6], *data_a.get_unchecked(2));
        perm_elt(a, b, 3, 2, 3, 0, 12, 9, c[5], *data_a.get_unchecked(3));
        perm_elt(a, b, 4, 3, 4, 1, 13, 10, c[4], *data_a.get_unchecked(4));
        perm_elt(a, b, 5, 4, 5, 2, 14, 11, c[3], *data_a.get_unchecked(5));
        perm_elt(a, b, 6, 5, 6, 3, 15, 12, c[2], *data_a.get_unchecked(6));
        perm_elt(a, b, 7, 6, 7, 4, 0, 13, c[1], *data_a.get_unchecked(7));
        perm_elt(a, b, 8, 7, 8, 5, 1, 14, c[0], *data_b.get_unchecked(0));
        perm_elt(a, b, 9, 8, 9, 6, 2, 15, c[15], *data_b.get_unchecked(1));
        perm_elt(a, b, 10, 9, 10, 7, 3, 0, c[14], *data_b.get_unchecked(2));
        perm_elt(a, b, 11, 10, 11, 8, 4, 1, c[13], *data_b.get_unchecked(3));
        perm_elt(a, b, 0, 11, 12, 9, 5, 2, c[12], *data_b.get_unchecked(4));
        perm_elt(a, b, 1, 0, 13, 10, 6, 3, c[11], *data_b.get_unchecked(5));
        perm_elt(a, b, 2, 1, 14, 11, 7, 4, c[10], *data_b.get_unchecked(6));
        perm_elt(a, b, 3, 2, 15, 12, 8, 5, c[9], *data_b.get_unchecked(7));
        perm_elt(a, b, 4, 3, 0, 13, 9, 6, c[8], *data_a.get_unchecked(0));
        perm_elt(a, b, 5, 4, 1, 14, 10, 7, c[7], *data_a.get_unchecked(1));
        perm_elt(a, b, 6, 5, 2, 15, 11, 8, c[6], *data_a.get_unchecked(2));
        perm_elt(a, b, 7, 6, 3, 0, 12, 9, c[5], *data_a.get_unchecked(3));
        perm_elt(a, b, 8, 7, 4, 1, 13, 10, c[4], *data_a.get_unchecked(4));
        perm_elt(a, b, 9, 8, 5, 2, 14, 11, c[3], *data_a.get_unchecked(5));
        perm_elt(a, b, 10, 9, 6, 3, 15, 12, c[2], *data_a.get_unchecked(6));
        perm_elt(a, b, 11, 10, 7, 4, 0, 13, c[1], *data_a.get_unchecked(7));
        perm_elt(a, b, 0, 11, 8, 5, 1, 14, c[0], *data_b.get_unchecked(0));
        perm_elt(a, b, 1, 0, 9, 6, 2, 15, c[15], *data_b.get_unchecked(1));
        perm_elt(a, b, 2, 1, 10, 7, 3, 0, c[14], *data_b.get_unchecked(2));
        perm_elt(a, b, 3, 2, 11, 8, 4, 1, c[13], *data_b.get_unchecked(3));
        perm_elt(a, b, 4, 3, 12, 9, 5, 2, c[12], *data_b.get_unchecked(4));
        perm_elt(a, b, 5, 4, 13, 10, 6, 3, c[11], *data_b.get_unchecked(5));
        perm_elt(a, b, 6, 5, 14, 11, 7, 4, c[10], *data_b.get_unchecked(6));
        perm_elt(a, b, 7, 6, 15, 12, 8, 5, c[9], *data_b.get_unchecked(7));
        perm_elt(a, b, 8, 7, 0, 13, 9, 6, c[8], *data_a.get_unchecked(0));
        perm_elt(a, b, 9, 8, 1, 14, 10, 7, c[7], *data_a.get_unchecked(1));
        perm_elt(a, b, 10, 9, 2, 15, 11, 8, c[6], *data_a.get_unchecked(2));
        perm_elt(a, b, 11, 10, 3, 0, 12, 9, c[5], *data_a.get_unchecked(3));
        perm_elt(a, b, 0, 11, 4, 1, 13, 10, c[4], *data_a.get_unchecked(4));
        perm_elt(a, b, 1, 0, 5, 2, 14, 11, c[3], *data_a.get_unchecked(5));
        perm_elt(a, b, 2, 1, 6, 3, 15, 12, c[2], *data_a.get_unchecked(6));
        perm_elt(a, b, 3, 2, 7, 4, 0, 13, c[1], *data_a.get_unchecked(7));
        perm_elt(a, b, 4, 3, 8, 5, 1, 14, c[0], *data_b.get_unchecked(0));
        perm_elt(a, b, 5, 4, 9, 6, 2, 15, c[15], *data_b.get_unchecked(1));
        perm_elt(a, b, 6, 5, 10, 7, 3, 0, c[14], *data_b.get_unchecked(2));
        perm_elt(a, b, 7, 6, 11, 8, 4, 1, c[13], *data_b.get_unchecked(3));
        perm_elt(a, b, 8, 7, 12, 9, 5, 2, c[12], *data_b.get_unchecked(4));
        perm_elt(a, b, 9, 8, 13, 10, 6, 3, c[11], *data_b.get_unchecked(5));
        perm_elt(a, b, 10, 9, 14, 11, 7, 4, c[10], *data_b.get_unchecked(6));
        perm_elt(a, b, 11, 10, 15, 12, 8, 5, c[9], *data_b.get_unchecked(7));
    }
}

#[inline(always)]
fn swap_bc(b: &mut [u32; 16], c: &mut [u32; 16]) {
    std::mem::swap(b, c);
}

#[inline(always)]
fn incr_w(w_low: &mut u32, w_high: &mut u32) {
    *w_low = w_low.wrapping_add(1);
    if *w_low == 0 {
        *w_high = w_high.wrapping_add(1);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const TEST_A_RESULT: [u8; 32] = [
        0xDA, 0x8F, 0x08, 0xC0, 0x2A, 0x67, 0xBA, 0x9A, 0x56, 0xBD, 0xD0, 0x79, 0x8E, 0x48, 0xAE,
        0x07, 0x14, 0x21, 0x5E, 0x09, 0x3B, 0x5B, 0x85, 0x06, 0x49, 0xA3, 0x77, 0x18, 0x99, 0x3F,
        0x54, 0xA2,
    ];
    const TEST_B_RESULT: [u8; 32] = [
        0xB4, 0x9F, 0x34, 0xBF, 0x51, 0x86, 0x4C, 0x30, 0x53, 0x3C, 0xC4, 0x6C, 0xC2, 0x54, 0x2B,
        0xDE, 0xC2, 0xF9, 0x6F, 0xD0, 0x6F, 0x5C, 0x53, 0x9A, 0xFF, 0x6E, 0xAD, 0x58, 0x83, 0xF7,
        0x32, 0x7A,
    ];
    const TEST_B_M1: [u32; 16] = [
        0x64636261, 0x68676665, 0x6C6B6A69, 0x706F6E6D, 0x74737271, 0x78777675, 0x302D7A79,
        0x34333231, 0x38373635, 0x42412D39, 0x46454443, 0x4A494847, 0x4E4D4C4B, 0x5251504F,
        0x56555453, 0x5A595857,
    ];
    const TEST_B_M2: [u32; 16] = [
        0x3231302D, 0x36353433, 0x2D393837, 0x64636261, 0x68676665, 0x6C6B6A69, 0x706F6E6D,
        0x74737271, 0x78777675, 0x00807A79, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
        0x00000000, 0x00000000,
    ];

    #[test]
    fn shabal256() {
        // test message A
        let test_data = [0u8; 64];
        let mut test_term = [0u32; 16];
        test_term[0] = 0x80;
        let hash_a = shabal256_hash_fast(&test_data, &test_term);
        assert_eq!(hash_a, TEST_A_RESULT);
        // test message B
        let hash_b = unsafe {
            shabal256_hash_fast(
                &std::mem::transmute::<[u32; 16], [u8; 64]>(TEST_B_M1),
                &TEST_B_M2,
            )
        };
        assert_eq!(hash_b, TEST_B_RESULT);
    }
}
