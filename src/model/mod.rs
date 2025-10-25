use std::sync::LazyLock;

use rand::{Rng, RngCore, SeedableRng, rngs::SmallRng};
use strum::{EnumIs, FromRepr, VariantNames};

pub mod attacks;
pub mod binary;
pub mod game;
pub mod mailbox;
pub mod movegen;
pub mod moving;
pub mod notation;

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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIs)]
#[repr(u8)]
pub enum Color {
    WHITE = 0,
    BLACK = 1,
}

impl Color {
    #[inline]
    fn opp(self) -> Self {
        unsafe { std::mem::transmute(self as u8 ^ 1) }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, EnumIs)]
#[repr(u8)]
pub enum Piece {
    #[default]
    NONE = 0,
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6,
}

impl From<Promotion> for Piece {
    fn from(value: Promotion) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Promotion {
    NONE = 0,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Castles {
    EAST = 0,
    WEST = 1,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum Special {
    #[default]
    NONE = 0,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    EAST = 6,
    WEST = 7,
}

impl From<Promotion> for Special {
    fn from(value: Promotion) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl TryFrom<Special> for Promotion {
    type Error = ();
    fn try_from(value: Special) -> Result<Self, Self::Error> {
        if value <= Special::QUEEN {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(())
        }
    }
}

impl From<Castles> for Special {
    fn from(value: Castles) -> Self {
        unsafe { std::mem::transmute(value as u8 + Self::EAST as u8) }
    }
}

impl TryFrom<Special> for Castles {
    type Error = ();
    fn try_from(value: Special) -> Result<Self, Self::Error> {
        if Special::EAST <= value {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(())
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rights(pub u8);
impl Rights {
    #[allow(unused)]
    const START: Rights = Rights(0b1111);
    #[allow(unused)]
    const NIL: Rights = Rights(0b0000);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PseudoLegal(BitMove);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Legal(BitMove);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitMove {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub special: Special,
    pub capture: Piece,
    pub attack: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransientInfo {
    pub ep_square: Option<Square>,
    pub halfmove_clock: u8,
    pub rights: Rights,
}

#[derive(Debug, Clone, Copy)]
pub struct BitBoard {
    pub pieces: [u64; 6],
    pub colors: [u64; 2],
    pub castling: &'static Castling,
    pub hash: u64,
    pub turn: u16,
    pub player: Color,
    pub trans: TransientInfo,
}

impl PartialEq for BitBoard {
    fn eq(&self, other: &Self) -> bool {
        self.pieces == other.pieces
            && self.colors == other.colors
            && self.hash == other.hash
            && self.player == other.player
            && self.trans.rights == other.trans.rights
            && self.trans.ep_square == other.trans.ep_square
    }
}

impl BitBoard {
    pub fn startpos() -> Self {
        let mut res = Self {
            pieces: [
                0x00FF_0000_0000_FF00,
                0x4200_0000_0000_0042,
                0x2400_0000_0000_0024,
                0x8100_0000_0000_0081,
                0x0800_0000_0000_0008,
                0x1000_0000_0000_0010,
            ],
            colors: [0x0000_0000_0000_FFFF, 0xFFFF_0000_0000_0000],
            castling: &CLASSIC_CASTLING,
            hash: 0,
            turn: 1,
            player: Color::WHITE,
            trans: TransientInfo {
                ep_square: None,
                halfmove_clock: 0,
                rights: Rights::START,
            },
        };
        res.hash = res.rehash();
        res
    }
}

#[derive(Debug)]
pub struct Castling {
    pub rook_move: [u64; 2],
    pub king_move: [u64; 2],
    pub safety: [u64; 2],
    pub space: [u64; 2],
    pub rook_from: [Square; 2],
    pub capture_own_rook: bool,
}

pub const CLASSIC_CASTLING: Castling = Castling {
    rook_move: [0xA000_0000_0000_00A0, 0x0900_0000_0000_0009],
    king_move: [0x5000_0000_0000_0050, 0x1400_0000_0000_0014],
    safety: [0x7000_0000_0000_0070, 0x1C00_0000_0000_001C],
    space: [0x6000_0000_0000_0060, 0x0E00_0000_0000_000E],
    rook_from: [Square::h1, Square::a1],
    capture_own_rook: false,
};

#[derive(Debug, Clone)]
pub struct ZobHasher {
    pub pieces: [[u64; 64]; 6],
    pub colors: [[u64; 64]; 2],
    pub ep_file: [u64; 8],
    pub castling: [u64; 4],
    pub black_to_move: u64,
}

impl ZobHasher {
    pub fn rng() -> SmallRng {
        SmallRng::from_seed(*b"3.141592653589793238462643383279")
    }

    pub fn new() -> Self {
        let mut pi = Self::rng();

        let mut pieces = [[0; 64]; 6];
        for piece in &mut pieces {
            pi.fill(&mut piece[..]);
        }

        let mut colors = [[0; 64]; 2];
        for color in &mut colors {
            pi.fill(&mut color[..]);
        }

        let mut eps = [0; 8];
        pi.fill(&mut eps[..]);

        let mut castling = [0; 4];
        pi.fill(&mut castling[..]);

        let black_to_move = pi.next_u64();

        ZobHasher {
            pieces,
            colors,
            ep_file: eps,
            castling,
            black_to_move,
        }
    }
}

pub static ZOBHASHER: LazyLock<ZobHasher> = LazyLock::new(ZobHasher::new);
