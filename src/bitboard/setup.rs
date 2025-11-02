/// # The 64-entry-array representaiton of a chessboard.
///
/// This is the simple and most obvious representation,
/// using a separate value in an array for each square, a so-called
/// 'board'-centric representation.
///
/// This module in particular is a generalized version allowing any
/// values, not just `Option<ChessMan>` to fill the squares.
///
/// In this library, the mailbox representation is only used
/// as a convenient and human-comprehendable way to initialize and
/// decode the bitboards used for actual computation. See
/// the [`bitboard`](`crate::model::bitboard`) module for details.
use strum::VariantArray;

use crate::{
    biterate,
    model::{
        ChessColor, ChessEchelon, ChessMan, Square, board::BitBoard, utils::SliceExtensions,
        vision::SimplePanopticon,
    },
};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct SimpleBoard<T>(pub [T; 64]);

impl<T> SimpleBoard<T> {
    /// Obtain a bit mask representing which squares the predicate
    /// returns true for.
    pub fn mask(&self, mut p: impl FnMut(Square, &T) -> bool) -> u64 {
        let mut res = 0;
        for ix in 0..=63 {
            if p(Square::from_u8(ix as u8), &self.0[ix]) {
                res |= 1 << ix;
            }
        }
        res
    }

    /// Write to a square
    pub fn set(&mut self, sq: Square, it: T) {
        self.0[sq.ix()] = it
    }

    /// Assign values to all squares for which a bit is set in the given mask.
    pub fn set_mask(&mut self, mask: u64, mut v: impl FnMut(Square) -> T) {
        biterate! {for sq in mask; {
            self.0[sq.ix()] = v(sq);
        }}
    }
}

impl SimpleBoard<Option<ChessMan>> {
    /// Set up a mailbox board from a bitboard.
    pub fn from_bitboard<BB: BitBoard>(bb: &BB) -> Self {
        let mut res = Self([None; 64]);

        for cm in ChessMan::VARIANTS.clones() {
            res.set_mask(bb.men(cm.col(), cm.ech()), |_| Some(cm));
        }

        res
    }

    pub fn as_bitboard<BB: BitBoard>(&self) -> BB {
        let mut bb = BB::empty();

        for cm in ChessMan::VARIANTS.clones() {
            bb.xor(cm.col(), cm.ech(), self.mask(|sq, x| x == &Some(cm)));
        }

        bb
    }
}
