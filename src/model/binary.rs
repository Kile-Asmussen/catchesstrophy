use std::simd::{num::SimdUint, u8x2, u64x2, u64x4};

use crate::model::{Square, notation::show_mask};

#[inline]
pub fn queen_diff_obs_simdx4(sq: Square, total: u64) -> u64 {
    let (neg_total, pos_total) = split(sq, total);
    let [rank, file] = rank_and_file(sq).to_array();
    let rays = u64x4::from_array([rank, file, diagonal(sq), antidiagonal(sq)]);
    diff_obs_simdx4(rays, neg_total, pos_total) & !(1 << sq as u8)
}

#[inline]
pub fn rook_diff_obs_simdx2(sq: Square, total: u64) -> u64 {
    let (neg_total, pos_total) = split(sq, total);
    diff_obs_simdx2(rank_and_file(sq), neg_total, pos_total) & !(1 << sq as u8)
}

#[inline]
pub fn bishop_diff_obs_simdx2(sq: Square, total: u64) -> u64 {
    let (neg_total, pos_total) = split(sq, total);
    diff_obs_simdx2(
        u64x2::from_array([diagonal(sq), antidiagonal(sq)]),
        neg_total,
        pos_total,
    ) & !(1 << sq as u8)
}

#[inline]
pub fn king_dumbfill_simdx4(mask: u64) -> u64 {
    let shift = u64x4::from_array([7, 8, 9, 1]);
    let wrap_up = u64x4::from_array([
        !0x8080_8080_8080_8080,
        !0,
        !0x0101_0101_0101_0101,
        !0x0101_0101_0101_0101,
    ]);
    let wrap_down = u64x4::from_array([
        !0x0101_0101_0101_0101,
        !0,
        !0x8080_8080_8080_8080,
        !0x8080_8080_8080_8080,
    ]);
    (u64x4::splat(mask) << shift & wrap_up | u64x4::splat(mask) >> shift & wrap_down).reduce_or()
}

#[inline]
pub fn knight_dumbfill_simdx4(mask: u64) -> u64 {
    let shift = u64x4::from_array([6, 15, 17, 10]);
    let wrap = u64x4::from_array([
        !0xC0C0_C0C0_C0C0_C0C0,
        !0x8080_8080_8080_8080,
        !0x0101_0101_0101_0101,
        !0x0303_0303_0303_0303,
    ]);
    (u64x4::splat(mask) << shift & wrap | u64x4::splat(mask) >> shift & wrap.reverse()).reduce_or()
}

#[inline]
fn diff_obs_1(ray: u64, neg_total: u64, pos_total: u64) -> u64 {
    let neg_hit = ray & neg_total;
    let pos_hit = ray & pos_total;
    let ms1b = 0x8000_0000_0000_0000 >> (neg_hit | 1).leading_zeros();
    let diff = pos_hit ^ pos_hit.wrapping_sub(ms1b);
    return ray & diff;
}

#[inline]
fn diff_obs_simdx2(ray: u64x2, neg_total: u64, pos_total: u64) -> u64 {
    let neg_hit = ray & u64x2::splat(neg_total);
    let pos_hit = ray & u64x2::splat(pos_total);
    let ms1b = u64x2::splat(0x8000_0000_0000_0000) >> (neg_hit | u64x2::splat(1)).leading_zeros();
    let diff = pos_hit ^ (pos_hit - ms1b);
    return (ray & diff).reduce_or();
}

#[inline]
fn diff_obs_simdx4(ray: u64x4, neg_total: u64, pos_total: u64) -> u64 {
    let neg_hit = ray & u64x4::splat(neg_total);
    let pos_hit = ray & u64x4::splat(pos_total);
    let ms1b = u64x4::splat(0x8000_0000_0000_0000) >> (neg_hit | u64x4::splat(1)).leading_zeros();
    let diff = pos_hit ^ (pos_hit - ms1b);
    return (ray & diff).reduce_or();
}

#[inline]
fn split(sq: Square, mask: u64) -> (u64, u64) {
    (mask & ((!0 >> 1) >> 63 - sq as u8), mask & (!1 << sq as u8))
}

#[inline]
fn rank_and_file(sq: Square) -> u64x2 {
    u64x2::from_array([0x0000_0000_0000_00FF, 0x0101_0101_0101_0101])
        << u64x2::from_array([sq as u64 & 0x38, sq as u64 & 0x7])
}

#[inline]
fn rank_row(sq: Square) -> u64 {
    0x0101_0101_0101_0101 << (sq as u8 & 0x7)
}

#[inline]
fn file_column(sq: Square) -> u64 {
    0x0000_0000_0000_00FF << (sq as u8 & 0x38)
}

#[inline]
fn both_diagonals(sq: Square) -> u64x2 {
    u64x2::from_array([0x8040_2010_0804_0201, 0x0102_0408_1020_4080])
}

#[inline]
fn diagonal(sq: Square) -> u64 {
    let sq = sq as u8;
    let n = 64 + (sq & 0x38) - ((sq << 3) & 0x38);
    (0x8040_2010_0804_0201u128 << n >> 64) as u64
}

#[inline]
fn antidiagonal(sq: Square) -> u64 {
    let sq = sq as u8;
    let n = 8 + (sq & 0x38) + ((sq << 3) & 0x38);
    (0x0102_0408_1020_4080u128 << n >> 64) as u64
}
