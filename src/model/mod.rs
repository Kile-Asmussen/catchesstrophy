use std::sync::LazyLock;

use rand::{Rng, RngCore, SeedableRng, rngs::SmallRng};
use strum::{EnumIs, FromRepr, VariantArray, VariantNames};

pub mod attacks;
pub mod binary;
pub mod bitboard;
pub mod game;
pub mod hash;
pub mod mailbox;
pub mod movegen;
pub mod moving;
pub mod notation;
pub mod utils;

/// Basic square enum
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
    FromRepr, VariantNames)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Square {
    a1 = 0o00, b1 = 0o01, c1 = 0o02, d1 = 0o03, e1 = 0o04, f1 = 0o05, g1 = 0o06, h1 = 0o07,
    a2 = 0100, b2 = 0o11, c2 = 0o12, d2 = 0o13, e2 = 0o14, f2 = 0o15, g2 = 0o16, h2 = 0o17,
    a3 = 0o20, b3 = 0o21, c3 = 0o22, d3 = 0o23, e3 = 0o24, f3 = 0o25, g3 = 0o26, h3 = 0o27,
    a4 = 0o30, b4 = 0o31, c4 = 0o32, d4 = 0o33, e4 = 0o34, f4 = 0o35, g4 = 0o36, h4 = 0o37,
    a5 = 0o40, b5 = 0o41, c5 = 0o42, d5 = 0o43, e5 = 0o44, f5 = 0o45, g5 = 0o46, h5 = 0o47,
    a6 = 0o50, b6 = 0o51, c6 = 0o52, d6 = 0o53, e6 = 0o54, f6 = 0o55, g6 = 0o56, h6 = 0o57,
    a7 = 0o60, b7 = 0o61, c7 = 0o62, d7 = 0o63, e7 = 0o64, f7 = 0o65, g7 = 0o66, h7 = 0o67,
    a8 = 0o70, b8 = 0o71, c8 = 0o72, d8 = 0o73, e8 = 0o74, f8 = 0o75, g8 = 0o76, h8 = 0o77,
}

impl Square {
    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }

    #[inline]
    pub fn from_u8(ix: u8) -> Self {
        unsafe { std::mem::transmute(ix & 0x3F) }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIs)]
#[repr(u8)]
pub enum Color {
    WHITE = 0,
    BLACK = 1,
}

impl Color {
    #[inline]
    pub fn opp(self) -> Self {
        unsafe { std::mem::transmute(self as u8 ^ 1) }
    }

    #[inline]
    pub fn sign(self) -> i8 {
        match self {
            Self::WHITE => 1,
            Self::BLACK => -1,
        }
    }

    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, VariantArray)]
#[repr(u8)]
pub enum ChessMan {
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6,
}

impl ChessMan {
    #[inline]
    fn ix(self) -> usize {
        self as usize - 1
    }
}

impl From<ChessPiece> for ChessMan {
    #[inline]
    fn from(value: ChessPiece) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<ChessPawn> for ChessMan {
    #[inline]
    fn from(value: ChessPawn) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<Promotion> for ChessMan {
    #[inline]
    fn from(value: Promotion) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<ChessCommoner> for ChessMan {
    #[inline]
    fn from(value: ChessCommoner) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ChessPawn {
    PAWN = 1,
}

impl ChessPawn {
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ChessPiece {
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6,
}

impl ChessPiece {
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ChessCommoner {
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
}

impl ChessCommoner {
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Promotion {
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
}

impl Promotion {
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Castles {
    EAST = 0,
    WEST = 1,
}

impl Castles {
    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Special {
    PAWN = 1,   // Double push or EPC
    KNIGHT = 2, // Promote
    BISHOP = 3, // Promote
    ROOK = 4,   // Promote
    QUEEN = 5,  // Promote
    EAST = 6,   // Castle
    WEST = 7,   // Castle
}

impl From<ChessPawn> for Special {
    #[inline]
    fn from(value: ChessPawn) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<Promotion> for Special {
    #[inline]
    fn from(value: Promotion) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<Castles> for Special {
    #[inline]
    fn from(value: Castles) -> Self {
        unsafe { std::mem::transmute(value as u8 + Special::EAST as u8) }
    }
}

impl From<ChessCommoner> for Special {
    #[inline]
    fn from(value: ChessCommoner) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
}

impl ChessPawn {
    fn from_special(special: Option<Special>) -> Option<Self> {
        if special == Some(Special::PAWN) {
            Some(ChessPawn::PAWN)
        } else {
            None
        }
    }
}

impl Promotion {
    fn from_special(special: Option<Special>) -> Option<Self> {
        let special = special?;
        if Special::KNIGHT <= special && special <= Special::QUEEN {
            Some(unsafe { std::mem::transmute(special) })
        } else {
            None
        }
    }
}

impl Castles {
    fn from_special(special: Option<Special>) -> Option<Self> {
        let special = special?;
        if Special::EAST <= special {
            Some(unsafe { std::mem::transmute(special as u8 - Special::EAST as u8) })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PseudoLegal(pub BitMove);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Legal(pub BitMove);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitMove {
    pub from: Square,
    pub to: Square,
    pub man: ChessMan,
    pub special: Option<Special>,
    pub capture: Option<ChessCommoner>,
}

impl BitMove {
    #[cfg(test)]
    pub fn sanity_check(self) {
        if Castles::from_special(self.special).is_some() {
            assert_eq!(self.man, ChessMan::KING);
            assert_eq!(self.capture, None);
            assert_eq!(self.from.rank(), self.to.rank());
        }

        if ChessPawn::from_special(self.special).is_some() {
            assert_eq!(self.man, ChessMan::PAWN);
            if self.capture.is_some() {
                assert_eq!(self.capture, Some(ChessCommoner::PAWN));
            } else {
                assert_eq!(self.from.ix().abs_diff(self.to.ix()), 16);
            }
        }
    }

    #[cfg(not(test))]
    pub fn sanity_check(self) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transients {
    pub en_passant: Option<EnPassant>,
    pub halfmove_clock: u8,
    pub rights: [[bool; 2]; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnPassant {
    square: Square,
    capture: Square,
}

#[derive(Debug)]
pub struct Castling {
    pub rook_move: [u64; 2],
    pub king_move: [u64; 2],
    pub safety: [u64; 2],
    pub space: [u64; 2],
    pub rook_from: [Square; 2],
    pub chess960: bool,
}

pub const CLASSIC_CASTLING: Castling = Castling {
    rook_move: [0xA000_0000_0000_00A0, 0x0900_0000_0000_0009],
    king_move: [0x5000_0000_0000_0050, 0x1400_0000_0000_0014],
    safety: [0x7000_0000_0000_0070, 0x1C00_0000_0000_001C],
    space: [0x6000_0000_0000_0060, 0x0E00_0000_0000_000E],
    rook_from: [Square::h1, Square::a1],
    chess960: false,
};
