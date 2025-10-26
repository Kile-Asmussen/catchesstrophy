use crate::model::{Color, Piece};

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
pub enum ColorPiece {
    BKING = -6,
    BQUEEN = -5,
    BROOK = -4,
    BBISHOP = -3,
    BKNIGHT = -2,
    BPAWN = -1,
    NONE = 0,
    WPAWN = 1,
    WKNIGHT = 2,
    WBISHOP = 3,
    WROOK = 4,
    WQUEEN = 5,
    WKING = 6,
}

impl ColorPiece {
    pub fn color(self) -> Color {
        unsafe { std::mem::transmute((self < Self::NONE) as u8) }
    }

    pub fn piece(self) -> Piece {
        unsafe { std::mem::transmute((self as i8).abs() as u8) }
    }
}

impl From<Piece> for ColorPiece {
    fn from(value: Piece) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<(Color, Piece)> for ColorPiece {
    fn from(value: (Color, Piece)) -> Self {
        unsafe { std::mem::transmute(value.1 as i8 * -(value.0 as i8)) }
    }
}
