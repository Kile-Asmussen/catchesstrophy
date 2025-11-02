use std::{borrow::Cow, marker::PhantomData};

use strum::VariantArray;

use crate::bitboard::{
    board::BitBoard,
    moving::clone_make_pseudolegal_move,
    utils::SliceExtensions,
    vision::{Panopticon, Vision},
};
use crate::model::*;

pub trait AttackMaskStrategy {
    type CachedData<'a, BB: BitBoard + 'a>: AttackMaskGenerator<'a, BB>;
    fn new<'a, BB: BitBoard>(board: &'a BB) -> Self::CachedData<'a, BB> {
        Self::CachedData::new(board)
    }
}

pub trait AttackMaskGenerator<'a, BB: BitBoard> {
    fn new(board: &'a BB) -> Self;

    fn attacks(&self, board: &'a BB, color: ChessColor) -> Attacks;

    fn attacks_after(&self, board: &'a BB, color: ChessColor, mv: ChessMove) -> Attacks;
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

pub struct FakeMoveSimplStrategy<X: Panopticon>(PhantomData<X>);
pub struct FakeMoveSimpleStrategyGenerator<BB: BitBoard, X: Panopticon>(PhantomData<(X, BB)>);

impl<X: Panopticon> AttackMaskStrategy for FakeMoveSimplStrategy<X> {
    type CachedData<'a, BB: BitBoard + 'a> = FakeMoveSimpleStrategyGenerator<BB, X>;
}

impl<'a, BB, X> AttackMaskGenerator<'a, BB> for FakeMoveSimpleStrategyGenerator<BB, X>
where
    BB: BitBoard + 'a,
    X: Panopticon,
{
    fn new(board: &'a BB) -> Self {
        FakeMoveSimpleStrategyGenerator(PhantomData)
    }

    fn attacks(&self, board: &BB, player: ChessColor) -> Attacks {
        let pan = X::new(board.total());
        match player {
            ChessColor::WHITE => Attacks {
                attack: attacks_from_echarray_white(pan, &board.side(ChessColor::WHITE)),
                targeted_king: board.men(ChessColor::BLACK, ChessPiece::KING),
            },
            ChessColor::BLACK => Attacks {
                attack: attacks_from_echarray_black(pan, &board.side(ChessColor::WHITE)),
                targeted_king: board.men(ChessColor::WHITE, ChessPiece::KING),
            },
        }
    }

    fn attacks_after(&self, board: &'a BB, color: ChessColor, mv: ChessMove) -> Attacks {
        let new_board = clone_make_pseudolegal_move(board, PseudoLegal(mv));
        Self::new(&new_board).attacks(&new_board, color)
    }
}

#[inline]
fn attacks_from_echarray_pieces<X: Panopticon>(pan: X, echs: &[u64; 6]) -> u64 {
    use ChessPiece::*;

    pan.knight().surveil(echs[KNIGHT.ix()])
        ^ pan.bishop().surveil(echs[BISHOP.ix()])
        ^ pan.rook().surveil(echs[ROOK.ix()])
        ^ pan.queen().surveil(echs[QUEEN.ix()])
        ^ pan.king().surveil(echs[KING.ix()])
}

#[inline]
fn attacks_from_echarray_black<X: Panopticon>(pan: X, echs: &[u64; 6]) -> u64 {
    pan.black_pawn().surveil(echs[ChessPiece::PAWN.ix()]) ^ attacks_from_echarray_pieces(pan, echs)
}

#[inline]
fn attacks_from_echarray_white<X: Panopticon>(pan: X, echs: &[u64; 6]) -> u64 {
    pan.white_pawn().surveil(echs[ChessPiece::PAWN.ix()]) ^ attacks_from_echarray_pieces(pan, echs)
}
