use std::sync::LazyLock;

use strum::{EnumIs, FromRepr, VariantNames};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
    EAST = 6,
    WEST = 7,
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

impl From<Castles> for Special {
    fn from(value: Castles) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[repr(u8)]
pub enum Rights {
    WHITE = 0b0011,
    BLACK = 0b1100,
    EAST = 0b0101,
    WEST = 0b1010,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordNotation {
    pub from: Square,
    pub to: Square,
    pub prom: Option<Promotion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIs)]
pub enum AlgNotaion {
    Pawn(AlgPawn, AlgCheck),
    Piece(AlgPiece, AlgCheck),
    OO(AlgCheck),
    OOO(AlgCheck),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlgPawn {
    pub from: Square,
    pub to: Square,
    pub capture: bool,
    pub promote: Promotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlgPiece {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub capture: bool,
    pub disambiguate: (bool, bool),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum AlgCheck {
    #[default]
    NONE = 0,
    CHECK = 1,
    MATE = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Pseudo(BitMove);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BitMove {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub special: Special,
    pub capture: Piece,
    pub captured: Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransientInfo {
    pub eps: Option<Square>,
    pub halfmove_clock: u8,
    pub castling_rights: u8,
}

pub struct BitBoard {
    pub piece: [u64; 6],
    pub color: [u64; 2],
    pub castling: &'static Castling,
    pub hash: u64,
    pub turn: u16,
    pub player: Color,
    pub trans: TransientInfo,
}

#[test]
fn test() {
    let mut x = BitBoard {
        piece: [0; 6],
        color: [0; 2],
        castling: &CLASSIC_CASTLING,
        hash: 0,
        turn: 1,
        player: Color::WHITE,
        trans: TransientInfo {
            eps: None,
            halfmove_clock: 0,
            castling_rights: 0b1111,
        },
    };

    x.make(BitMove {
        from: Square::a1,
        to: Square::h8,
        piece: Piece::ROOK,
        special: Special::NONE,
        capture: Piece::NONE,
        captured: Square::h8,
    });
}

impl BitBoard {
    pub fn make(&mut self, mv: BitMove) -> TransientInfo {
        let res = self.trans;
        self.turn += self.player as u16;

        self.simple_move(mv);
        self.promotion_move(mv);
        self.castling_move(mv);

        self.player = self.player.opp();
        self.hash ^= ZOBHASHER.black_to_move;
        self.update_transient(mv);
        res
    }

    fn simple_move(&mut self, mv: BitMove) {
        if mv.special > Special::EAST {
            return;
        }

        let bits = (1 << mv.from as u8) | (1 << mv.to as u8);
        let cap = ((mv.capture != Piece::NONE) as u64) << mv.captured as u8;

        self.piece[mv.piece as usize - 1] ^= bits;
        self.piece[(mv.capture as usize).saturating_sub(1)] ^= cap;
        self.color[self.player as usize] ^= bits;
        self.color[self.player.opp() as usize] ^= cap;

        self.hash ^= ZOBHASHER.pieces[mv.piece as usize][mv.from as usize];
        self.hash ^= ZOBHASHER.pieces[mv.piece as usize][mv.to as usize];
        self.hash ^= ZOBHASHER.pieces[mv.capture as usize][mv.captured as usize];
        self.hash ^= ZOBHASHER.color[self.player as usize][mv.from as usize];
        self.hash ^= ZOBHASHER.color[self.player as usize][mv.to as usize];
        // ??? can we cmov this somehow?
        self.hash ^= ZOBHASHER.color[self.player.opp() as usize][mv.captured as usize];
    }

    fn promotion_move(&mut self, mv: BitMove) {
        todo!()
    }

    fn castling_move(&mut self, mv: BitMove) {
        todo!()
    }

    fn update_transient(&mut self, mv: BitMove) {
        todo!()
    }
}

pub struct Castling {
    pub rook_move: [u64; 2],
    pub king_move: [u64; 2],
    pub safety: [u64; 2],
    pub space: [u64; 2],
    pub rook_from: [Square; 2],
    pub rook_to: [Square; 2],
    pub king_from: Square,
    pub king_to: [Square; 2],
    pub capture_own_rook: bool,
}

pub const CLASSIC_CASTLING: Castling = Castling {
    rook_move: [0; 2],
    king_move: [0; 2],
    safety: [0; 2],
    space: [0; 2],
    rook_from: [Square::h1, Square::a1],
    rook_to: [Square::d1, Square::f1],
    king_from: Square::e1,
    king_to: [Square::g1, Square::c1],
    capture_own_rook: false,
};

pub struct ZobHasher {
    pub pieces: [[u64; 64]; 7],
    pub color: [[u64; 64]; 2],
    pub eps: [u64; 8],
    pub black_to_move: u64,
}

pub static ZOBHASHER: LazyLock<ZobHasher> = LazyLock::new(|| todo!());
