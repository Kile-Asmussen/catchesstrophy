use std::{hash::Hasher, ops::BitXor, sync::LazyLock};

use chrono::format::Colons;
use rand::{Rng, RngCore, SeedableRng, rngs::SmallRng};
use static_init::Lazy;
use strum::VariantArray;

use crate::model::{Castles, ChessMan, Color, EnPassant, Square, attacks::PieceVision};

pub trait ZobristDetails {
    fn hash_en_passant(&self, ep: Option<EnPassant>) -> u64;
    fn hash_rights(&self, rights: [[bool; 2]; 2]) -> u64;
    fn black(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct DefaultZobristDetails {
    pub ep_files: [u64; 8],
    pub rights: [[u64; 2]; 2],
    pub black_to_move: u64,
}

impl DefaultZobristDetails {
    fn new(rng: &mut SmallRng) -> DefaultZobristDetails {
        let mut ep_files = [0; 8];
        rng.fill(&mut ep_files[..]);

        let rights = [[rng.next_u64(), rng.next_u64()], [rng.next_u64(), rng.next_u64()]];

        let black_to_move = rng.next_u64();

        Self {
            ep_files,
            rights,
            black_to_move,
        }
    }
}

impl ZobristDetails for DefaultZobristDetails {
    #[inline]
    fn hash_en_passant(&self, ep: Option<EnPassant>) -> u64 {
        if let Some(ep) = ep {
            self.ep_files[ep.capture.ix() & 0x7]
        } else {
            0
        }
    }

    #[inline]
    fn hash_rights(&self, rights: [[bool; 2]; 2]) -> u64 {
        let mut res = 0;
        for c in [Color::WHITE, Color::BLACK] {
            for d in [Castles::EAST, Castles::WEST] {
                res ^= self.rights[c.ix()][d.ix()];
            }
        }
        res
    }

    #[inline]
    fn black(&self) -> u64 {
        self.black_to_move
    }
}

trait HasDefaultZobristDetails {
    fn default_details(&self) -> &DefaultZobristDetails;
}

impl<ZT: HasDefaultZobristDetails> ZobristDetails for ZT {
    #[inline]
    fn hash_en_passant(&self, ep: Option<EnPassant>) -> u64 {
        self.default_details().hash_en_passant(ep)
    }

    #[inline]
    fn hash_rights(&self, rights: [[bool; 2]; 2]) -> u64 {
        self.default_details().hash_rights(rights)
    }

    #[inline]
    fn black(&self) -> u64 {
        self.default_details().black()
    }
}

pub trait ZobristTables: ZobristDetails + 'static {
    fn static_table() -> &'static Self;
    fn hash_full(&self, masks: &[[u64; 6]; 2]) -> u64;
    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64;
    fn hash_move(&self, player: Color, man: ChessMan, bits: u64) -> u64;
    fn hash_square(&self, player: Color, man: ChessMan, sq: Square) -> u64;
    fn hash_castling(&self, player: Color, king_bits: u64, rook_bits: u64) -> u64;
}

pub fn pi_rng() -> SmallRng {
    SmallRng::from_seed(*b"3.141592653589793238462643383279")
}

#[derive(Debug, Clone)]
pub struct CompactZobristTables {
    pub men: [[u64; 64]; 6],
    pub colors: [[u64; 64]; 2],
    pub details: DefaultZobristDetails,
}

impl CompactZobristTables {
    pub fn new() -> Self {
        let mut pi = pi_rng();

        let mut men = [[0; 64]; 6];
        for piece in &mut men {
            pi.fill(&mut piece[..]);
        }

        let mut colors = [[0; 64]; 2];
        for color in &mut colors {
            pi.fill(&mut color[..]);
        }

        CompactZobristTables {
            men,
            colors,
            details: DefaultZobristDetails::new(&mut pi),
        }
    }

    #[inline]
    fn hash_color_mask(&self, color: Color, mut mask: u64) -> u64 {
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            let bit = 1 << sq;
            mask ^= bit;
            res ^= self.colors[color.ix()][sq as usize & 0x3F];
        }
        res
    }

    #[inline]
    fn hash_man_mask(&self, man: ChessMan, mut mask: u64) -> u64 {
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            let bit = 1 << sq;
            mask ^= bit;
            res ^= self.men[man.ix()][sq as usize & 0x3F];
        }
        res
    }
}

impl HasDefaultZobristDetails for CompactZobristTables {
    fn default_details(&self) -> &DefaultZobristDetails {
        &self.details
    }
}

static COMPACT_ZOBRIST: LazyLock<CompactZobristTables> = LazyLock::new(CompactZobristTables::new);

impl ZobristTables for CompactZobristTables {
    fn static_table() -> &'static Self {
        &COMPACT_ZOBRIST
    }

    fn hash_move(&self, player: Color, man: ChessMan, bits: u64) -> u64 {
        self.hash_color_mask(player, bits) ^ self.hash_man_mask(man, bits)
    }

    fn hash_square(&self, player: Color, man: ChessMan, sq: Square) -> u64 {
        self.men[man.ix()][sq.ix()] ^ self.colors[player.ix()][sq.ix()]
    }

    fn hash_castling(&self, player: Color, king_bits: u64, rook_bits: u64) -> u64 {
        self.hash_man_mask(ChessMan::KING, king_bits)
            ^ self.hash_man_mask(ChessMan::ROOK, rook_bits)
            ^ self.hash_color_mask(player, king_bits | rook_bits)
    }

    fn hash_full(&self, masks: &[[u64; 6]; 2]) -> u64 {
        let mut res = 0;
        res ^= self.hash_color_mask(Color::WHITE, bin_sum(&masks[Color::WHITE.ix()]));
        res ^= self.hash_color_mask(Color::BLACK, bin_sum(&masks[Color::BLACK.ix()]));

        for man in ChessMan::VARIANTS {
            res ^= self.hash_man_mask(
                *man,
                masks[Color::WHITE.ix()][man.ix()] | masks[Color::WHITE.ix()][man.ix()],
            );
        }

        res
    }

    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64 {
        let mut res = 0;
        for man in ChessMan::VARIANTS {
            res ^= self.hash_man_mask(*man, men[man.ix()]);
        }

        for color in [Color::WHITE, Color::BLACK] {
            res ^= self.hash_color_mask(color, colors[color.ix()]);
        }

        res
    }
}

pub fn bin_sum<const N: usize>(data: &[u64; N]) -> u64 {
    let mut res = 0;
    for i in 0..N {
        res |= data[i];
    }
    res
}

#[derive(Debug, Clone)]
pub struct FullZobristTables {
    pub masks: [[[u64; 64]; 6]; 2],
    pub details: DefaultZobristDetails,
}

impl HasDefaultZobristDetails for FullZobristTables {
    fn default_details(&self) -> &DefaultZobristDetails {
        &self.details
    }
}

impl FullZobristTables {
    pub fn new() -> Self {
        let mut pi = pi_rng();

        let mut masks = [[[0; 64]; 6]; 2];
        for pieces in &mut masks {
            for piece in pieces {
                pi.fill(&mut piece[..]);
            }
        }

        FullZobristTables {
            masks,
            details: DefaultZobristDetails::new(&mut pi),
        }
    }

    #[inline]
    fn hash_mask(&self, color: Color, man: ChessMan, mut mask: u64) -> u64 {
        let mut res = 0;
        let board = self.masks[color.ix()][man.ix()];
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            let bit = 1 << sq;
            mask ^= bit;
            res ^= board[sq as usize & 0x3F];
        }
        res
    }
}

impl ZobristTables for FullZobristTables {
    fn static_table() -> &'static Self {
        todo!()
    }

    #[inline]
    fn hash_move(&self, player: Color, man: ChessMan, bits: u64) -> u64 {
        self.hash_mask(player, man, bits)
    }

    #[inline]
    fn hash_square(&self, player: Color, man: ChessMan, sq: Square) -> u64 {
        self.masks[player.ix()][man.ix()][sq.ix()]
    }

    #[inline]
    fn hash_castling(&self, player: Color, king_bits: u64, rook_bits: u64) -> u64 {
        self.hash_mask(player, ChessMan::KING, king_bits)
            ^ self.hash_mask(player, ChessMan::ROOK, rook_bits)
    }

    #[inline]
    fn hash_full(&self, masks: &[[u64; 6]; 2]) -> u64 {
        let mut res = 0;
        for c in [Color::WHITE, Color::BLACK] {
            for m in ChessMan::VARIANTS {
                res ^= self.hash_mask(c, *m, masks[c.ix()][m.ix()]);
            }
        }
        res
    }

    #[inline]
    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64 {
        let mut res = 0;
        for c in [Color::WHITE, Color::BLACK] {
            for m in ChessMan::VARIANTS {
                res ^= self.hash_mask(c, *m, colors[c.ix()] & men[m.ix()]);
            }
        }
        res
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoHashes;

impl ZobristDetails for NoHashes {
    fn hash_en_passant(&self, ep: Option<EnPassant>) -> u64 {
        0
    }

    fn hash_rights(&self, rights: [[bool; 2]; 2]) -> u64 {
        0
    }

    fn black(&self) -> u64 {
        0
    }
}

impl ZobristTables for NoHashes {
    fn static_table() -> &'static Self {
        &NoHashes
    }

    fn hash_full(&self, masks: &[[u64; 6]; 2]) -> u64 {
        0
    }

    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64 {
        0
    }

    fn hash_move(&self, player: Color, man: ChessMan, bits: u64) -> u64 {
        0
    }

    fn hash_square(&self, player: Color, man: ChessMan, sq: Square) -> u64 {
        0
    }

    fn hash_castling(&self, player: Color, king_bits: u64, rook_bits: u64) -> u64 {
        0
    }
}

///////////////////////////
// For HashMap ////////////
///////////////////////////

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ZobHasher(pub u64);

impl Hasher for ZobHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {}

    fn write_u64(&mut self, i: u64) {
        self.0 ^= i;
    }
}
