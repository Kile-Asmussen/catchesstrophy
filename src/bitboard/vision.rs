use std::marker::PhantomData;

use crate::{
    biterate,
    model::{
        ChessColor, ChessPiece, Square,
        binary::{
            bishop_diff_obs_simdx2, black_pawn_advance_fill, black_pawn_attack_fill,
            black_pawn_attack_fill_simdx2, king_dumbfill_simdx4, knight_dumbfill_simdx4,
            queen_diff_obs_simdx4, rook_diff_obs_simdx2, white_pawn_advance_fill,
            white_pawn_attack_fill, white_pawn_attack_fill_simdx2,
        },
    },
};

pub type MostlyBits = SimplePanopticon<
    PawnsBitBlit<true>,
    PawnsBitBlit<false>,
    KnightDumbfill,
    FastObsDiffBishop,
    FastObsDiffRook,
    FastObsDiffQueen,
    KingDumbfill,
>;

#[derive(Debug, Clone, Copy)]
pub struct SimplePanopticon<WhitePawn, BlackPawn, Knight, Bishop, Rook, Queen, King>(
    u64,
    PhantomData<(WhitePawn, BlackPawn, Knight, Bishop, Rook, Queen, King)>,
)
where
    WhitePawn: PawnVision,
    BlackPawn: PawnVision,
    Knight: PieceVision,
    Bishop: PieceVision,
    Rook: PieceVision,
    Queen: PieceVision,
    King: PieceVision;

pub trait Panopticon: Clone + Copy {
    fn new(total: u64) -> Self;
    fn white_pawn(&self) -> impl PawnVision;
    fn black_pawn(&self) -> impl PawnVision;
    fn knight(&self) -> impl PieceVision;
    fn bishop(&self) -> impl PieceVision;
    fn rook(&self) -> impl PieceVision;
    fn queen(&self) -> impl PieceVision;
    fn king(&self) -> impl PieceVision;
}

impl<WhitePawn, BlackPawn, Knight, Bishop, Rook, Queen, King> Panopticon
    for SimplePanopticon<WhitePawn, BlackPawn, Knight, Bishop, Rook, Queen, King>
where
    WhitePawn: PawnVision,
    BlackPawn: PawnVision,
    Knight: PieceVision,
    Bishop: PieceVision,
    Rook: PieceVision,
    Queen: PieceVision,
    King: PieceVision,
{
    fn new(total: u64) -> Self {
        Self(total, PhantomData)
    }

    #[inline]
    fn white_pawn(&self) -> impl PawnVision {
        WhitePawn::new(self.0)
    }

    #[inline]
    fn black_pawn(&self) -> impl PawnVision {
        BlackPawn::new(self.0)
    }

    #[inline]
    fn knight(&self) -> impl PieceVision {
        Knight::new(self.0)
    }

    #[inline]
    fn bishop(&self) -> impl PieceVision {
        Bishop::new(self.0)
    }

    #[inline]
    fn rook(&self) -> impl PieceVision {
        Rook::new(self.0)
    }

    #[inline]
    fn queen(&self) -> impl PieceVision {
        Queen::new(self.0)
    }

    #[inline]
    fn king(&self) -> impl PieceVision {
        King::new(self.0)
    }
}

pub trait Vision: Copy + Clone {
    fn new(total: u64) -> Self;

    #[inline]
    fn see(self, sq: Square) -> u64 {
        self.surveil(1 << sq as u8)
    }

    #[inline]
    fn surveil(self, mask: u64) -> u64 {
        let mut res = 0;
        biterate! {for sq in mask; {
            res |= self.see(sq);
        }}
        res
    }
}

pub trait PieceVision: Vision {
    #[inline]
    fn hits(self, sq: Square, friendly: u64) -> u64 {
        self.see(sq) & !friendly
    }

    const ID: ChessPiece;
}

pub trait PawnVision: Vision {
    #[inline]
    fn hits(self, sq: Square, enemy_and_eps: u64) -> u64 {
        self.see(sq) & enemy_and_eps
    }

    #[inline]
    fn push(self, sq: Square) -> u64 {
        self.advance(1 << sq as u8)
    }

    #[inline]
    fn advance(self, mask: u64) -> u64 {
        let mut res = 0;
        biterate! {for sq in mask; {
            res |= self.push(sq);
        }}
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
        Self(total)
    }

    #[inline]
    fn see(self, sq: Square) -> u64 {
        rook_diff_obs_simdx2(sq, self.0)
    }
}

impl PieceVision for FastObsDiffRook {
    const ID: ChessPiece = ChessPiece::ROOK;
}

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

impl PieceVision for FastObsDiffBishop {
    const ID: ChessPiece = ChessPiece::BISHOP;
}

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

impl PieceVision for FastObsDiffQueen {
    const ID: ChessPiece = ChessPiece::QUEEN;
}

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

impl PieceVision for KnightDumbfill {
    const ID: ChessPiece = ChessPiece::KNIGHT;
}

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
        king_dumbfill_simdx4(mask)
    }
}

impl PieceVision for KingDumbfill {
    const ID: ChessPiece = ChessPiece::KING;
}
