use std::sync::LazyLock;

use rand::{Rng, RngCore, SeedableRng, rngs::SmallRng};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Squares(u64);

impl Iterator for Squares {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let n = self.0.trailing_zeros();
            self.0 &= !(1 << n);
            Square::from_repr(n as u8 & 0x3F)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
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

impl From<Castles> for Special {
    fn from(value: Castles) -> Self {
        unsafe { std::mem::transmute(value as u8 + Self::EAST as u8) }
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

#[test]
fn test() {
    let mut x = BitBoard {
        pieces: [0; 6],
        colors: [0; 2],
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

    x.simple_move(BitMove {
        from: Square::a1,
        to: Square::h8,
        piece: Piece::ROOK,
        special: Special::NONE,
        capture: Piece::NONE,
        attack: Square::h8,
    });

    println!("{:?}", x);
}

impl BitBoard {
    pub fn startpos() {}

    pub fn rehash(&self) -> u64 {
        use Color::*;
        use Piece::*;

        let mut res = 0;

        for piece in [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING] {
            let board = &ZOBHASHER.pieces[piece as usize - 1];
            for sq in Squares(self.pieces[piece as usize - 1]) {
                res ^= board[sq as usize]
            }
        }

        for color in [WHITE, BLACK] {
            let board = &ZOBHASHER.colors[color as usize - 1];
            for sq in Squares(self.colors[color as usize - 1]) {
                res ^= board[sq as usize]
            }
        }

        for ix in [0, 1, 2, 3] {
            if (self.trans.rights.0 & 1 << ix) != 0 {
                res ^= ZOBHASHER.castling[ix];
            }
        }

        if let Some(sq) = self.trans.ep_square {
            res ^= ZOBHASHER.ep_file[sq as usize & 0x7];
        }

        if self.player == BLACK {
            res ^= ZOBHASHER.black_to_move;
        }

        res
    }

    pub fn make_move(&mut self, mv: BitMove) -> TransientInfo {
        let res = self.trans;
        self.turn += self.player as u16;

        self.simple_move(mv);
        self.promotion_move(mv);
        self.castling_move(mv);

        self.update_transient(mv);

        self.turn += (self.player == Color::BLACK) as u16;
        self.player = self.player.opp();
        self.hash ^= ZOBHASHER.black_to_move;

        res
    }

    pub fn unmake_move(&mut self, mv: BitMove, trans: TransientInfo) {}

    #[inline]
    fn simple_move(&mut self, mv: BitMove) {
        if Special::EAST <= mv.special {
            return;
        }

        let piece = (mv.piece as usize).saturating_sub(1);
        let bits = (1 << mv.from as u8) | (1 << mv.to as u8);
        let is_cap = !mv.capture.is_none();
        let cap_piece = (mv.capture as usize).saturating_sub(1);
        let cap_bit = 1 << mv.attack as u8;
        let cap_sq = mv.attack as usize;
        let player = self.player as usize;
        let opponent = self.player.opp() as usize;
        let from = mv.from as usize;
        let to = mv.to as usize;

        self.pieces[piece] ^= bits;
        self.colors[player] ^= bits;
        self.colors[opponent] ^= cap_bit;

        self.hash ^= ZOBHASHER.pieces[piece][from];
        self.hash ^= ZOBHASHER.pieces[piece][to];
        self.hash ^= ZOBHASHER.colors[player][from];
        self.hash ^= ZOBHASHER.colors[player][to];

        if is_cap {
            self.pieces[cap_piece] ^= cap_bit;
            self.hash ^= ZOBHASHER.pieces[cap_piece][cap_sq];
            self.hash ^= ZOBHASHER.colors[opponent][cap_sq];
        }
    }

    #[inline]
    fn promotion_move(&mut self, mv: BitMove) {
        if mv.special < Special::KNIGHT || Special::QUEEN < mv.special {
            return;
        }

        let pawn = Piece::PAWN as usize;
        let piece = (mv.piece as usize).saturating_sub(1);
        let bit = 1 << mv.to as u8;
        let to = mv.to as usize;

        self.pieces[pawn] ^= bit;
        self.pieces[piece] ^= bit;

        self.hash ^= ZOBHASHER.pieces[pawn][to];
        self.hash ^= ZOBHASHER.pieces[piece][to];
    }

    #[inline]
    fn castling_move(&mut self, mv: BitMove) {
        if mv.special < Special::EAST {
            return;
        }

        let dir = mv.special as usize - Special::EAST as usize;
        let rank = 0xFF << if self.player.is_black() { 56 } else { 0 };
        let king = Piece::KING as usize - 1;
        let king_move = self.castling.king_move[dir] & rank;
        let rook = Piece::ROOK as usize - 1;
        let rook_move = self.castling.rook_move[dir] & rank;
        let player = self.player as usize;

        self.pieces[king] ^= king_move;
        self.pieces[rook] ^= rook_move;
        self.colors[player] ^= king_move;
        self.colors[player] ^= rook_move;

        self.hash ^= ZOBHASHER.pieces[king][self.castling.king_from as usize];
        self.hash ^= ZOBHASHER.pieces[king][self.castling.king_to[dir] as usize];
        self.hash ^= ZOBHASHER.pieces[rook][self.castling.rook_from[dir] as usize];
        self.hash ^= ZOBHASHER.pieces[rook][self.castling.rook_to[dir] as usize];
    }

    #[inline]
    fn update_transient(&mut self, mv: BitMove) {
        let player = self.player as usize;
        let opponent = self.player.opp() as usize;

        if mv.piece == Piece::KING {
            let ix = player << 1;
            let bits = 0x3 << ix;
            self.trans.rights.0 &= !bits;
            self.hash ^= ZOBHASHER.castling[ix];
            self.hash ^= ZOBHASHER.castling[ix + 1];
        }

        if mv.capture == Piece::ROOK {
            for dir in [Castles::EAST, Castles::WEST] {
                let dir = dir as usize;
                if mv.attack == self.castling.rook_from[dir] {
                    let ix = dir + (opponent << 1);
                    let bit = 1 << ix;
                    self.trans.rights.0 &= !bit;
                    self.hash ^= ZOBHASHER.castling[ix];
                }
            }
        }

        if mv.capture != Piece::NONE || mv.piece == Piece::PAWN {
            self.trans.halfmove_clock = 0;
        }

        if let Some(ep_square) = self.trans.ep_square {
            let ep_ix = ep_square as u8;
            self.hash ^= ZOBHASHER.ep_file[(ep_ix & 0x7) as usize];
        }

        if mv.piece == Piece::PAWN && (mv.from as u8).abs_diff(mv.to as u8) == 16 {
            let ep_ix = (mv.from as u8).min(mv.to as u8) + 8;
            self.trans.ep_square = Square::from_repr(ep_ix);
            self.hash ^= ZOBHASHER.ep_file[(ep_ix & 0x7) as usize];
        } else {
            self.trans.ep_square = None;
        }
    }
}

#[derive(Debug)]
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
    rook_move: [0xA0 | 0xA0 << 56, 0x09 | 0x09 << 56],
    king_move: [0x50 | 0x50 << 56, 0x14 | 0x14 << 56],
    safety: [0x70 | 0x70 << 56, 0x1C | 0x1C << 56],
    space: [0x60 | 0x60 << 56, 0x0E | 0x0E << 56],
    rook_from: [Square::h1, Square::a1],
    rook_to: [Square::d1, Square::f1],
    king_from: Square::e1,
    king_to: [Square::g1, Square::c1],
    capture_own_rook: false,
};

fn pi_rng() -> SmallRng {
    SmallRng::from_seed(*b"3.141592653589793238462643383279")
}

#[derive(Debug, Clone)]
pub struct ZobHasher {
    pub pieces: [[u64; 64]; 6],
    pub colors: [[u64; 64]; 2],
    pub ep_file: [u64; 8],
    pub castling: [u64; 4],
    pub black_to_move: u64,
}

impl ZobHasher {
    fn new() -> Self {
        let mut pi = pi_rng();

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
