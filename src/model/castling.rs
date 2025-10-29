use crate::model::Square;

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
