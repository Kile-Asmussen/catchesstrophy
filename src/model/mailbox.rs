use crate::model::{ChessMan, Color};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Mailbox<T>([T; 64]);

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
