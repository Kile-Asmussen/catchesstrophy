use crate::model::*;

#[derive(Debug)]
pub struct BitCastling {
    pub rook_move: [u64; 2],
    pub king_move: [u64; 2],
    pub safety: [u64; 2],
    pub space: [u64; 2],
    pub back_rank: [u64; 2],
    pub rules: CastlingRules,
}

impl BitCastling {
    pub const STANDARD: BitCastling = BitCastling {
        rook_move: [0xA000_0000_0000_00A0, 0x0900_0000_0000_0009],
        king_move: [0x5000_0000_0000_0050, 0x1400_0000_0000_0014],
        safety: [0x7000_0000_0000_0070, 0x1C00_0000_0000_001C],
        space: [0x6000_0000_0000_0060, 0x0E00_0000_0000_000E],
        back_rank: [0x0000_0000_0000_00FF, 0xFF00_0000_0000_0000],
        rules: CastlingRules::STANDARD,
    };
}
