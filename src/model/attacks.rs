use std::marker::PhantomData;

use crate::model::{
    BitBoard, Color, Square,
    binary::{
        bishop_diff_obs_simdx2, knight_dumbfill_simdx4, queen_diff_obs_simdx4, rook_diff_obs_simdx2,
    },
};

struct Panopticon<
    WP: PawnVision,
    BP: PawnVision,
    N: PieceVision,
    B: PieceVision,
    R: PieceVision,
    Q: PieceVision,
    K: PieceVision,
>(WP, BP, N, B, R, Q, K);

impl<
    WP: PawnVision,
    BP: PawnVision,
    N: PieceVision,
    B: PieceVision,
    R: PieceVision,
    Q: PieceVision,
    K: PieceVision,
> Panopticon<WP, BP, N, B, R, Q, K>
{
    pub fn new(total: u64) -> Self {
        Self(
            WP::new(total),
            BP::new(total),
            N::new(total),
            B::new(total),
            R::new(total),
            Q::new(total),
            K::new(total),
        )
    }
}

trait Vision: Copy + Clone {
    fn new(total: u64) -> Self;

    #[inline]
    fn see(self, sq: Square) -> u64 {
        self.surveil(1 << sq as u8)
    }

    #[inline]
    fn surveil(self, mut mask: u64) -> u64 {
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros() as u8;
            let bit = 1 << sq;
            mask ^= bit;
            res |= self.see(unsafe { std::mem::transmute(sq & 0x63) });
        }
        res
    }
}

trait PieceVision: Vision {
    #[inline]
    fn hits(self, sq: Square, friendly: u64) -> u64 {
        self.see(sq) & !friendly
    }
}

trait PawnVision: Vision {
    fn hits(self, sq: Square, enemy_and_eps: u64) -> u64 {
        self.see(sq) & enemy_and_eps
    }

    fn push(self, sq: Square) -> u64 {
        self.advance(1 << sq as u8)
    }

    fn advance(self, mut mask: u64) -> u64 {
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros() as u8;
            let bit = 1 << sq;
            mask ^= bit;
            res |= self.push(unsafe { std::mem::transmute(sq & 0x63) });
        }
        res
    }
}

#[derive(Clone, Copy, Debug, Hash)]
#[repr(transparent)]
struct PawnsBitBlit<const SHL: bool>(pub u64);

impl<const SHL: bool> Vision for PawnsBitBlit<SHL> {
    fn new(total: u64) -> Self {
        Self(!total)
    }

    #[inline]
    fn surveil(self, mask: u64) -> u64 {
        if SHL {
            (mask << 7 & 0x0101_0101_0101_0101) | (mask << 9 & 0x8080_8080_8080_8080)
        } else {
            (mask >> 7 & 0x8080_8080_8080_8080) | (mask >> 9 & 0x0101_0101_0101_0101)
        }
    }
}

impl<const SHL: bool> PawnVision for PawnsBitBlit<SHL> {
    #[inline]
    fn advance(self, mask: u64) -> u64 {
        if SHL {
            mask << 8 & self.0 | (((mask & 0x0000_0000_0000_FF00) << 8) & self.0) << 8 & self.0
        } else {
            mask >> 8 & self.0 | (((mask & 0x00FF_0000_0000_0000) >> 8) & self.0) >> 8 & self.0
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct FastObsDiffRook(pub u64);

impl Vision for FastObsDiffRook {
    fn new(total: u64) -> Self {
        Self(total)
    }

    fn see(self, sq: Square) -> u64 {
        rook_diff_obs_simdx2(sq, self.0)
    }
}

impl PieceVision for FastObsDiffRook {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct FastObsDiffBishop(pub u64);

impl Vision for FastObsDiffBishop {
    fn new(total: u64) -> Self {
        Self(total)
    }

    fn see(self, sq: Square) -> u64 {
        bishop_diff_obs_simdx2(sq, self.0)
    }
}

impl PieceVision for FastObsDiffBishop {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct FastObsDiffQueen(pub u64);

impl Vision for FastObsDiffQueen {
    fn new(total: u64) -> Self {
        Self(total)
    }

    fn see(self, sq: Square) -> u64 {
        queen_diff_obs_simdx4(sq, self.0)
    }
}

impl PieceVision for FastObsDiffQueen {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct KnightDumbfill;

impl Vision for KnightDumbfill {
    fn new(total: u64) -> Self {
        Self
    }

    fn surveil(self, mut mask: u64) -> u64 {
        knight_dumbfill_simdx4(mask)
    }
}

impl PieceVision for KnightDumbfill {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct KingDumbfill;

impl Vision for KingDumbfill {
    fn new(total: u64) -> Self {
        Self
    }

    fn surveil(self, mut mask: u64) -> u64 {
        knight_dumbfill_simdx4(mask)
    }
}

impl PieceVision for KingDumbfill {}
