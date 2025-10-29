use std::marker::PhantomData;

use crate::model::{
    ChessColor, Square,
    binary::{
        bishop_diff_obs_simdx2, black_pawn_advance_fill, black_pawn_attack_fill,
        black_pawn_attack_fill_simdx2, knight_dumbfill_simdx4, queen_diff_obs_simdx4,
        rook_diff_obs_simdx2, white_pawn_advance_fill, white_pawn_attack_fill,
        white_pawn_attack_fill_simdx2,
    },
};

type DefaultChessMen = ChessMen<
    PawnsBitBlit<true>,
    PawnsBitBlit<false>,
    KnightDumbfill,
    FastObsDiffBishop,
    FastObsDiffRook,
    FastObsDiffQueen,
    KingDumbfill,
>;

pub struct ChessMen<WP, BP, N, B, R, Q, K>(u64, PhantomData<(WP, BP, N, B, R, Q, K)>)
where
    WP: PawnVision,
    BP: PawnVision,
    N: PieceVision,
    B: PieceVision,
    R: PieceVision,
    Q: PieceVision,
    K: PieceVision;

pub trait Panopticon {
    fn new(total: u64) -> Self;
    fn white_pawn(&self) -> impl PawnVision;
    fn black_pawn(&self) -> impl PawnVision;
    fn knight(&self) -> impl PieceVision;
    fn bishop(&self) -> impl PieceVision;
    fn rook(&self) -> impl PieceVision;
    fn queen(&self) -> impl PieceVision;
    fn king(&self) -> impl PieceVision;
}

impl<WP, BP, N, B, R, Q, K> Panopticon for ChessMen<WP, BP, N, B, R, Q, K>
where
    WP: PawnVision,
    BP: PawnVision,
    N: PieceVision,
    B: PieceVision,
    R: PieceVision,
    Q: PieceVision,
    K: PieceVision,
{
    fn new(total: u64) -> Self {
        Self(total, PhantomData)
    }

    #[inline]
    fn white_pawn(&self) -> impl PawnVision {
        WP::new(self.0)
    }

    #[inline]
    fn black_pawn(&self) -> impl PawnVision {
        WP::new(self.0)
    }

    #[inline]
    fn knight(&self) -> impl PieceVision {
        N::new(self.0)
    }

    #[inline]
    fn bishop(&self) -> impl PieceVision {
        B::new(self.0)
    }

    #[inline]
    fn rook(&self) -> impl PieceVision {
        R::new(self.0)
    }

    #[inline]
    fn queen(&self) -> impl PieceVision {
        R::new(self.0)
    }

    #[inline]
    fn king(&self) -> impl PieceVision {
        K::new(self.0)
    }
}

pub trait Vision: Copy + Clone {
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
            res |= self.see(unsafe { std::mem::transmute(sq & 0x3F) });
        }
        res
    }
}

pub trait PieceVision: Vision {
    #[inline]
    fn hits(self, sq: Square, friendly: u64) -> u64 {
        self.see(sq) & !friendly
    }
}

pub trait PawnVision: Vision {
    #[inline]
    fn hits(self, sq: Square, enemy_and_eps: u64) -> u64 {
        self.see(sq) & enemy_and_eps | self.push(sq)
    }

    #[inline]
    fn push(self, sq: Square) -> u64 {
        self.advance(1 << sq as u8)
    }

    #[inline]
    fn advance(self, mut mask: u64) -> u64 {
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros() as u8;
            let bit = 1 << sq;
            mask ^= bit;
            res |= self.push(unsafe { std::mem::transmute(sq & 0x3F) });
        }
        res
    }
}

#[derive(Clone, Copy, Debug, Hash)]
#[repr(transparent)]
pub struct PawnsBitBlit<const SHL: bool>(u64);

impl<const SHL: bool> Vision for PawnsBitBlit<SHL> {
    #[inline]
    fn new(total: u64) -> Self {
        Self(!total)
    }

    #[inline]
    fn surveil(self, mask: u64) -> u64 {
        if SHL {
            white_pawn_attack_fill(mask)
        } else {
            black_pawn_attack_fill(mask)
        }
    }
}

impl<const SHL: bool> PawnVision for PawnsBitBlit<SHL> {
    #[inline]
    fn advance(self, mask: u64) -> u64 {
        if SHL {
            white_pawn_advance_fill(mask, self.0)
        } else {
            black_pawn_advance_fill(mask, self.0)
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct FastObsDiffRook(u64);

impl Vision for FastObsDiffRook {
    #[inline]
    fn new(total: u64) -> Self {
        Self(!total)
    }

    #[inline]
    fn see(self, sq: Square) -> u64 {
        rook_diff_obs_simdx2(sq, self.0)
    }
}

impl PieceVision for FastObsDiffRook {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct FastObsDiffBishop(u64);

impl Vision for FastObsDiffBishop {
    #[inline]
    fn new(total: u64) -> Self {
        Self(total)
    }

    #[inline]
    fn see(self, sq: Square) -> u64 {
        bishop_diff_obs_simdx2(sq, self.0)
    }
}

impl PieceVision for FastObsDiffBishop {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct FastObsDiffQueen(u64);

impl Vision for FastObsDiffQueen {
    #[inline]
    fn new(total: u64) -> Self {
        Self(total)
    }

    #[inline]
    fn see(self, sq: Square) -> u64 {
        queen_diff_obs_simdx4(sq, self.0)
    }
}

impl PieceVision for FastObsDiffQueen {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct KnightDumbfill;

impl Vision for KnightDumbfill {
    #[inline]
    fn new(total: u64) -> Self {
        Self
    }

    #[inline]
    fn surveil(self, mut mask: u64) -> u64 {
        knight_dumbfill_simdx4(mask)
    }
}

impl PieceVision for KnightDumbfill {}

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct KingDumbfill;

impl Vision for KingDumbfill {
    #[inline]
    fn new(total: u64) -> Self {
        Self
    }

    #[inline]
    fn surveil(self, mut mask: u64) -> u64 {
        knight_dumbfill_simdx4(mask)
    }
}

impl PieceVision for KingDumbfill {}
