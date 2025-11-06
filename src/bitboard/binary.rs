/// Efficient bit arithmetic on chessboards.
///
/// Here be ~~dragons~~ SIMD instructions.
use std::simd::{num::SimdUint, u64x2, u64x4};

use crate::bitboard::vision::{PawnVision, PawnsBitBlit, Vision};

use crate::model::*;

/// Compute all the squares attacked by a queen using the [`diff_obs_simdx4`] function.
///
/// - `sq` the square of the queen
/// - `total` the occupancy of the chessboard
///
/// This produces the attacked squares, to get legal moves one must mask out the friendly
/// pieces.
#[inline]
pub fn queen_diff_obs_simdx4(sq: Square, total: u64) -> u64 {
    let (neg_total, pos_total) = split(sq, total);
    let [rank, file] = rank_and_file(sq).to_array();
    let rays = u64x4::from_array([rank, file, diagonal(sq), antidiagonal(sq)]);
    diff_obs_simdx4(rays, neg_total, pos_total) & !(1 << sq as u8)
}

/// Compute all the squares attacked by a rook using the [`diff_obs_simdx2`] function.
///
/// - `sq` the square of the rook
/// - `total` the occupancy of the chessboard
///
/// This produces the attacked squares, to get legal moves one must mask out the friendly
/// pieces.
#[inline]
pub fn rook_diff_obs_simdx2(sq: Square, total: u64) -> u64 {
    let (neg_total, pos_total) = split(sq, total);
    diff_obs_simdx2(rank_and_file(sq), neg_total, pos_total) & !(1 << sq as u8)
}

/// Compute all the squares attacked by a bishop using the [`diff_obs_simdx2`] function.
///
/// - `sq` the square of the bishop
/// - `total` the occupancy of the chessboard
///
/// This produces the attacked squares, to get legal moves one must mask out the friendly
/// pieces.
#[inline]
pub fn bishop_diff_obs_simdx2(sq: Square, total: u64) -> u64 {
    let (neg_total, pos_total) = split(sq, total);
    diff_obs_simdx2(
        u64x2::from_array([diagonal(sq), antidiagonal(sq)]),
        neg_total,
        pos_total,
    ) & !(1 << sq as u8)
}

/// Compute all squares attacked by white pawns at once using bit operations.
#[inline]
pub fn white_pawn_attack_fill(mask: u64) -> u64 {
    mask << 7 & !0x8080_8080_8080_8080 | mask << 9 & !0x0101_0101_0101_0101
}

/// Compute all squares attacked by white pawns at once using bit operations, in
/// parallel using simd operations. Probably not faster than [`white_pawn_attack_fill`].
#[inline]
pub fn white_pawn_attack_fill_simdx2(mask: u64) -> u64 {
    (u64x2::splat(mask) << u64x2::from_array([7, 9])
        & u64x2::from_array([!0x8080_8080_8080_8080, !0x0101_0101_0101_0101]))
    .reduce_or()
}

/// Advance all white pawns at once, using bit operations.
#[inline]
pub fn white_pawn_advance_fill(mask: u64, empty: u64) -> u64 {
    mask << 8 & empty | ((mask & 0x0000_0000_0000_FF00) << 8 & empty) << 8 & empty
}

/// Compute all squares attacked by black pawns at once using bit operations.
#[inline]
pub fn black_pawn_attack_fill(mask: u64) -> u64 {
    mask >> 7 & !0x0101_0101_0101_0101 | mask >> 9 & !0x8080_8080_8080_8080
}

/// Compute all squares attacked by black pawns at once using bit operations, in
/// parallel using simd operations. Probably not faster than [`black_pawn_attack_fill`].
#[inline]
pub fn black_pawn_attack_fill_simdx2(mask: u64) -> u64 {
    (u64x2::splat(mask) >> u64x2::from_array([7, 9])
        & u64x2::from_array([!0x0101_0101_0101_0101, !0x8080_8080_8080_8080]))
    .reduce_or()
}

/// Advance all black pawns at once, using bit operations.
#[inline]
pub fn black_pawn_advance_fill(mask: u64, empty: u64) -> u64 {
    mask >> 8 & empty | ((mask & 0x00FF_0000_0000_0000) >> 8 & empty) >> 8 & empty
}

/// Computes all 8 directions any number of kings can move in two operations,
/// using SIMD instructions.
///
/// This is essentially one step of a bit-level flood fill algorithm.
#[inline]
pub fn king_dumbfill_simdx4(mask: u64) -> u64 {
    let shift = u64x4::from_array([7, 8, 9, 1]);
    let wrap_shl = u64x4::from_array([
        !0x8080_8080_8080_8080,
        !0,
        !0x0101_0101_0101_0101,
        !0x0101_0101_0101_0101,
    ]);
    let wrap_slr = u64x4::from_array([
        !0x0101_0101_0101_0101,
        !0,
        !0x8080_8080_8080_8080,
        !0x8080_8080_8080_8080,
    ]);
    (u64x4::splat(mask) << shift & wrap_shl | u64x4::splat(mask) >> shift & wrap_slr).reduce_or()
}

/// Computes all 8 directions a knight can move in two operations, using SIMD instructions.
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

/// The dumb7fill algorithm for rooks.
///
/// The simplest way of determining the legal moves of a rook
/// is to loop over all the squares in each cardinal direction until
/// a blocker is encountered.
///
/// This algorithm does just that, but on a bit-by-bit level, for every
/// rook in parallel, in all four directions in parallel.
///
/// Furthermore the maximum nunber of times this process needs to be
/// performed is 7, so this algorithm does away with loop logic entirely
/// and just does it 7 times.
///
/// It is _very_ fast.
#[inline]
pub fn rook_dumb7fill_simdx2(rooks: u64, empty: u64) -> u64 {
    const SHIFT: u64x2 = u64x2::from_array([1, 8]);

    const WRAP_SHL: u64x2 = u64x2::from_array([!0x0101_0101_0101_0101, !0]);
    let empty_shl = u64x2::splat(empty) & WRAP_SHL;
    let mut rooks_shl = u64x2::splat(rooks);
    let mut flood_shl = u64x2::splat(0);
    for _ in 0..5 {
        rooks_shl = rooks_shl << SHIFT & empty_shl;
        flood_shl |= rooks_shl;
    }
    flood_shl |= rooks_shl << SHIFT & empty_shl;
    flood_shl = flood_shl << SHIFT & WRAP_SHL;

    const WRAP_SHR: u64x2 = u64x2::from_array([!0x8080_8080_8080_8080, !0]);
    let empty_shr = u64x2::splat(empty) & WRAP_SHR;
    let mut rooks_shr = u64x2::splat(rooks);
    let mut flood_shr = u64x2::splat(0);
    for _ in 0..5 {
        rooks_shr = rooks_shr >> SHIFT & empty_shr;
        flood_shr |= rooks_shr;
    }
    flood_shr |= rooks_shr >> SHIFT & empty_shr;
    flood_shr = flood_shr >> SHIFT & WRAP_SHR;

    return (flood_shl | flood_shr).reduce_or();
}

/// The dumb7fill algorithm for bishops.
#[inline]
pub fn bishop_dumb7fill_simdx2(bishops: u64, empty: u64) -> u64 {
    const SHIFT: u64x2 = u64x2::from_array([7, 9]);
    const WRAP: u64x2 = u64x2::from_array([!0x8080_8080_8080_8080, !0x0101_0101_0101_0101]);

    let empty_shl = u64x2::splat(empty) & WRAP;
    let mut rooks_shl = u64x2::splat(bishops);
    let mut flood_shl = u64x2::splat(0);
    for _ in 0..5 {
        rooks_shl = rooks_shl << SHIFT & empty_shl;
        flood_shl |= rooks_shl;
    }
    flood_shl |= rooks_shl << SHIFT & empty_shl;
    flood_shl = flood_shl << SHIFT & WRAP;

    let empty_shr = u64x2::splat(empty) & WRAP.reverse();
    let mut rooks_shr = u64x2::splat(bishops);
    let mut flood_shr = u64x2::splat(0);
    for _ in 0..5 {
        rooks_shr = rooks_shr >> SHIFT & empty_shr;
        flood_shr |= rooks_shr;
    }
    flood_shr |= rooks_shr >> SHIFT & empty_shr;
    flood_shr = flood_shr >> SHIFT & WRAP.reverse();

    return (flood_shl | flood_shr).reduce_or();
}

/// The dumb7fill algorithm for queens (and optionally bishops and rooks).
#[inline]
pub fn queen_dumb7fill_simdx2(queens: u64, rooks: u64, bishops: u64, empty: u64) -> u64 {
    const SHIFT: u64x4 = u64x4::from_array([8, 7, 9, 1]);
    const WRAP_SHL: u64x4 = u64x4::from_array([
        !0,
        !0x8080_8080_8080_8080,
        !0x0101_0101_0101_0101,
        !0x0101_0101_0101_0101,
    ]);
    const WRAP_SHR: u64x4 = u64x4::from_array([
        !0,
        !0x8080_8080_8080_8080,
        !0x0101_0101_0101_0101,
        !0x0101_0101_0101_0101,
    ]);

    let empty_shl = u64x4::splat(empty) & WRAP_SHL;
    let mut queens_shl =
        u64x4::from_array([rooks | queens, bishops | queens, bishops | queens, rooks | queens]);
    let mut flood_shl = u64x4::splat(0);
    for _ in 0..5 {
        queens_shl = queens_shl << SHIFT & empty_shl;
        flood_shl |= queens_shl;
    }
    flood_shl |= queens_shl << SHIFT & empty_shl;
    flood_shl = flood_shl << SHIFT & WRAP_SHL;

    let empty_shr = u64x4::splat(empty) & WRAP_SHR;
    let mut queens_shr =
        u64x4::from_array([rooks | queens, bishops | queens, bishops | queens, rooks | queens]);
    let mut flood_shr = u64x4::splat(0);
    for _ in 0..5 {
        queens_shr = queens_shr >> SHIFT & empty_shr;
        flood_shr |= queens_shr;
    }
    flood_shr |= queens_shr >> SHIFT & empty_shr;
    flood_shr = flood_shr >> SHIFT & WRAP_SHR;

    return (flood_shl | flood_shr).reduce_or();
}

/// Obstruction difference.
///
/// An algorithm using the wrapping behavior of 2's-compliment subtraction
/// to efficiently compute the first blockers of the as sliding piece.
///
/// Must be called twice for rooks and bishops, and four times for queens.
///
/// - `neg_ray` the sliding ray pointing away from the origin square in
///   a negative direction (see [`CompassRose`](crate::model::CompassRose)).
/// - `pos_ray` the sliding ray pointing away from the origin square in
///   a positive direction.
/// - `total` the total occupancy mask of the chessboard.
///
/// The result is the attack mask. To obtain the legal move mask, one must
/// remove same-colored pieces.
#[inline]
fn obs_diff(neg_ray: u64, pos_ray: u64, total: u64) -> u64 {
    let neg_hit = neg_ray & total;
    let pos_hit = pos_ray & total;
    let ms1b = 0x8000_0000_0000_0000 >> (neg_hit | 1).leading_zeros();
    let diff = pos_hit ^ pos_hit.wrapping_sub(ms1b);
    return (neg_ray | pos_ray) & diff;
}

/// Difference of obstructions.
///
/// A personal refinement of the obstruction difference algorithm (see [`obs_diff`])
/// which relies on splitting the occupancy mask instead of the ray.
///
/// The result is the attack mask _including the square itself_ which must be masked out.
#[inline]
fn diff_obs(ray: u64, neg_total: u64, pos_total: u64) -> u64 {
    let neg_hit = ray & neg_total;
    let pos_hit = ray & pos_total;
    let ms1b = 0x8000_0000_0000_0000 >> (neg_hit | 1).leading_zeros();
    let diff = pos_hit ^ pos_hit.wrapping_sub(ms1b);
    return ray & diff;
}

/// Difference of obstruction computed on two rays simultaneously using SIMD.
///
/// Useful for rooks and bishops.
#[inline]
fn diff_obs_simdx2(ray: u64x2, neg_total: u64, pos_total: u64) -> u64 {
    let neg_hit = ray & u64x2::splat(neg_total);
    let pos_hit = ray & u64x2::splat(pos_total);
    let ms1b = u64x2::splat(0x8000_0000_0000_0000) >> (neg_hit | u64x2::splat(1)).leading_zeros();
    let diff = pos_hit ^ (pos_hit - ms1b);
    return (ray & diff).reduce_or();
}

/// Difference of obstruction computed on four rays simultaneously using SIMD.
///
/// Useful for queens. Requires AVX2-capable hardware to work well.
#[inline]
fn diff_obs_simdx4(ray: u64x4, neg_total: u64, pos_total: u64) -> u64 {
    let neg_hit = ray & u64x4::splat(neg_total);
    let pos_hit = ray & u64x4::splat(pos_total);
    let ms1b = u64x4::splat(0x8000_0000_0000_0000) >> (neg_hit | u64x4::splat(1)).leading_zeros();
    let diff = pos_hit ^ (pos_hit - ms1b);
    return (ray & diff).reduce_or();
}

/// Splits a mask in two, zeroing the bits above and below a given square, respectively.
///
/// Used for the [`diff_obs`] family of functions.
#[inline]
fn split(sq: Square, mask: u64) -> (u64, u64) {
    (mask & ((!0 >> 1) >> 63 - sq as u8), mask & (!1 << sq as u8))
}

/// Computes both the `rank` and `file` functions in parallel using SIMD.
#[inline]
fn rank_and_file(sq: Square) -> u64x2 {
    u64x2::from_array([0x0000_0000_0000_00FF, 0x0101_0101_0101_0101])
        << u64x2::from_array([sq as u64 & 0x38, sq as u64 & 0x7])
}

/// Computes the mask of the rank intersecting a square using a bit shift.
#[inline]
fn rank_row(sq: Square) -> u64 {
    0x0101_0101_0101_0101 << (sq as u8 & 0x7)
}

/// Computes the mask of the file intersecting a square using a bit shift.
#[inline]
fn file_column(sq: Square) -> u64 {
    0x0000_0000_0000_00FF << (sq as u8 & 0x38)
}

/// Computes the diagonal (south west-north east) intersecting
/// a square, using a u128 bit shift.
#[inline]
fn diagonal(sq: Square) -> u64 {
    let sq = sq as u8;
    let n = 64 + (sq & 0x38) - ((sq << 3) & 0x38);
    (0x8040_2010_0804_0201u128 << n >> 64) as u64
}

/// Computes the diagonal (south east-north west) intersecting
/// a square, using a u128 bit shift.
#[inline]
fn antidiagonal(sq: Square) -> u64 {
    let sq = sq as u8;
    let n = 8 + (sq & 0x38) + ((sq << 3) & 0x38);
    (0x0102_0408_1020_4080u128 << n >> 64) as u64
}
