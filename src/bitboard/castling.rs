use crate::model::{CastlingDirection, ChessColor, Square};

#[derive(Debug)]
pub struct Castling {
    pub rook_move: [u64; 2],
    pub king_move: [u64; 2],
    pub safety: [u64; 2],
    pub space: [u64; 2],
    pub back_rank: [u64; 2],
    pub rook_start: [[Square; 2]; 2],
    pub king_end: [[Square; 2]; 2],
    pub king_start: [Square; 2],
    pub chess960: bool,
}

pub const CLASSIC_CASTLING: Castling = Castling {
    rook_move: [0xA000_0000_0000_00A0, 0x0900_0000_0000_0009],
    king_move: [0x5000_0000_0000_0050, 0x1400_0000_0000_0014],
    safety: [0x7000_0000_0000_0070, 0x1C00_0000_0000_001C],
    space: [0x6000_0000_0000_0060, 0x0E00_0000_0000_000E],
    back_rank: [0x0000_0000_0000_00FF, 0xFF00_0000_0000_0000],
    rook_start: [[Square::h1, Square::a1], [Square::h8, Square::a8]],
    king_start: [Square::e1, Square::e8],
    king_end: [[Square::g1, Square::c1], [Square::g8, Square::c8]],
    chess960: false,
};
