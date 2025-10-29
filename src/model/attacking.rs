use std::borrow::Cow;

use strum::VariantArray;

use crate::model::{
    BitMove, ChessEchelon, PseudoLegal,
    bitboard::BitBoard,
    moving::clone_make_pseudolegal_move,
    utils::SliceExtensions,
    vision::{Panopticon, Vision},
};

use super::ChessColor;

pub trait AttackMaskStrategy<'a>: Sized {
    fn new<BB: BitBoard>(board: &'a BB) -> Self;

    fn attacks<BB: BitBoard, X: Panopticon>(&self, board: &'a BB, color: ChessColor) -> Attacks;

    fn attacks_after<BB: BitBoard, X: Panopticon>(
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

pub struct FakeMoveAttackMaskStrategy<'a>(Cow<'a, [u64; 6]>);

impl<'a> AttackMaskStrategy<'a> for FakeMoveAttackMaskStrategy<'a> {
    fn attacks<BB: BitBoard, X: Panopticon>(&self, board: &BB, player: ChessColor) -> Attacks {
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

    fn new<BB: BitBoard>(board: &'a BB) -> Self {
        Self(board.side(board.ply().0))
    }

    fn attacks_after<BB: BitBoard, X: Panopticon>(
        &self,
        board: &'a BB,
        color: ChessColor,
        mv: BitMove,
    ) -> Attacks {
        let new_board = clone_make_pseudolegal_move(board, PseudoLegal(mv));
        FakeMoveAttackMaskStrategy::new(&new_board).attacks::<BB, X>(&new_board, color)
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
