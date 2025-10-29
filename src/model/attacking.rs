use std::{borrow::Cow, marker::PhantomData};

use strum::VariantArray;

use crate::model::{
    BitMove, ChessEchelon, PseudoLegal,
    bitboard::BitBoard,
    moving::clone_make_pseudolegal_move,
    utils::SliceExtensions,
    vision::{Panopticon, Vision},
};

use super::ChessColor;

pub trait AttackMaskStrategy {
    type CachedMasks<'a, BB: BitBoard + 'a>: AttackMaskGenerator<'a, BB>;
    fn new<'a, BB: BitBoard>(board: &'a BB) -> Self::CachedMasks<'a, BB> {
        Self::CachedMasks::new(board)
    }
}

pub trait AttackMaskGenerator<'a, BB: BitBoard> {
    fn new(board: &'a BB) -> Self;

    fn attacks<X: Panopticon>(&self, board: &'a BB, color: ChessColor) -> Attacks;

    fn attacks_after<X: Panopticon>(
        &self,
        board: &'a BB,
        color: ChessColor,
        mv: BitMove,
    ) -> Attacks;
}

#[derive(Debug, Clone, Copy)]
pub struct Attacks {
    pub attack: u64,
    pub targeted_king: u64,
}

impl Attacks {
    pub fn check(self) -> bool {
        (self.attack & self.targeted_king) != 0
    }
}

pub struct FakeMoveEcharrayStrategy;
pub struct FakeMoveEcharrayStrategyGenerator<'a, BB: BitBoard + 'a>(
    Cow<'a, [u64; 6]>,
    PhantomData<BB>,
);

impl AttackMaskStrategy for FakeMoveEcharrayStrategy {
    type CachedMasks<'a, BB: BitBoard + 'a> = FakeMoveEcharrayStrategyGenerator<'a, BB>;
}

impl<'a, BB: BitBoard> AttackMaskGenerator<'a, BB> for FakeMoveEcharrayStrategyGenerator<'a, BB> {
    fn new(board: &'a BB) -> Self {
        FakeMoveEcharrayStrategyGenerator(board.side(board.ply().0), PhantomData)
    }

    fn attacks<X: Panopticon>(&self, board: &BB, player: ChessColor) -> Attacks {
        let pan = X::new(board.total());
        match player {
            ChessColor::WHITE => Attacks {
                attack: attacks_from_echarray_white(pan, &self.0),
                targeted_king: board.men(ChessColor::BLACK, ChessEchelon::KING),
            },
            ChessColor::BLACK => Attacks {
                attack: attacks_from_echarray_black(pan, &self.0),
                targeted_king: board.men(ChessColor::WHITE, ChessEchelon::KING),
            },
        }
    }

    fn attacks_after<X: Panopticon>(
        &self,
        board: &'a BB,
        color: ChessColor,
        mv: BitMove,
    ) -> Attacks {
        let new_board = clone_make_pseudolegal_move(board, PseudoLegal(mv));
        FakeMoveEcharrayStrategyGenerator::new(&new_board).attacks::<X>(&new_board, color)
    }
}

#[inline]
fn attacks_from_echarray_pieces<X: Panopticon>(pan: X, echs: &[u64; 6]) -> u64 {
    use ChessEchelon::*;

    pan.knight().surveil(echs[KNIGHT.ix()])
        ^ pan.bishop().surveil(echs[BISHOP.ix()])
        ^ pan.rook().surveil(echs[ROOK.ix()])
        ^ pan.queen().surveil(echs[QUEEN.ix()])
        ^ pan.king().surveil(echs[KING.ix()])
}

#[inline]
fn attacks_from_echarray_black<X: Panopticon>(pan: X, echs: &[u64; 6]) -> u64 {
    pan.black_pawn().surveil(echs[ChessEchelon::PAWN.ix()])
        ^ attacks_from_echarray_pieces(pan, echs)
}

#[inline]
fn attacks_from_echarray_white<X: Panopticon>(pan: X, echs: &[u64; 6]) -> u64 {
    pan.white_pawn().surveil(echs[ChessEchelon::PAWN.ix()])
        ^ attacks_from_echarray_pieces(pan, echs)
}
