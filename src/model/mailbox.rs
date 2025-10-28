use strum::VariantArray;

use crate::model::{ChessMan, Color, attacks::ChessMen, bitboard::BitBoard};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Mailbox<T>(pub [T; 64]);

impl<T: Clone + Copy> Mailbox<T> {
    pub fn mask(&self, mut p: impl FnMut(&T) -> bool) -> u64 {
        let mut res = 0;
        for i in 0..=63 {
            if p(&self.0[i]) {
                res |= 1 << i;
            }
        }
        res
    }

    pub fn set(&mut self, mut mask: u64, mut v: impl FnMut() -> T) {
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            mask ^= 1 << sq;
            self.0[sq as usize & 0x3F] = v();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, VariantArray)]
#[repr(i8)]
pub enum ChessSet {
    BKING = -6,
    BQUEEN = -5,
    BROOK = -4,
    BBISHOP = -3,
    BKNIGHT = -2,
    BPAWN = -1,
    WPAWN = 1,
    WKNIGHT = 2,
    WBISHOP = 3,
    WROOK = 4,
    WQUEEN = 5,
    WKING = 6,
}

impl ChessSet {
    fn man(self) -> ChessMan {
        unsafe { std::mem::transmute((self as i8).abs()) }
    }

    fn color(self) -> Color {
        if (self as i8) < 0 {
            Color::BLACK
        } else {
            Color::WHITE
        }
    }
}

impl Mailbox<Option<ChessSet>> {
    pub fn from_bitboard<BB: BitBoard>(bb: &BB) -> Self {
        let mut res = Self([None; 64]);

        for cm in ChessSet::VARIANTS {
            res.set(bb.mask(cm.color(), cm.man()), || Some(*cm));
        }

        res
    }
}
