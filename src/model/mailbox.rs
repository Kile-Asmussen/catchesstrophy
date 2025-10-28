use crate::model::{ChessMan, Color};

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

    pub fn set(&mut self, ix: u64, mut v: impl FnMut() -> T) {
        for i in 0..=63 {
            if (ix & 1 << i) != 0 {
                self.0[i] = v();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
