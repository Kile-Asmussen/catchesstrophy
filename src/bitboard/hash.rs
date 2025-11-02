//! Hashing of chess positions.
//!
//! Zobrist hashing is a technique for efficiently hashing game
//! states, used in a variety of perfect information games played
//! on a grid, such as Chess, Shōgi, Draughts (Checkers), Go, and
//! their variants.
//!
//! The technique functions by pre-generating random numbers for each
//! field on the board and each piece that might occupy that field,
//! and indeed every mutually exclusive component of the entire
//! game state, and then selecting the appropriate random values
//! and combining them using the exclusive-or (XOR, `^`) operation.
//!
//! This has several benefits: first of all it is extremely efficient
//! on mordenr hardware, and second, the exclusive-or operation forms an
//! [abelian group] on fixed-size binary numbers, and forms an [involution].
//!
//! [abelian group]: https://en.wikipedia.org/wiki/Abelian_group
//! [involution]: https://en.wikipedia.org/wiki/Involution_(mathematics)
//!
//! This means that when a game state changes, a change hash exists
//! equal to the XOR-sum of the previous state's hash and the current
//! state's hash. This change-hash, called a delta in this codebase, can
//! be in almost all cases be computed directly from the description of
//! the move that changes the game state. Thus the hash of the current
//! game state is the XOR-sum of the delta hashes of every move made so
//! far in the game history, together with the hash of the initial position.
//!
//! The main drawback of Zobrist hashing is that the tables of random values
//! are comparatively large. A distinct random value is needed for chessman that
//! can occupy every square on a chess board. A naive implementation thus uses
//! 2 × 6 × 64 = 756 random values to hash just the board state.
//!
//! As with all hashing, Zobrist hashing will always have collisions, as there
//! are approximately 4.8×10<sup>44</sup> legal chess positions reachable from
//! the standard starting position, while there are only 2<sup>64</sup> possible
//! values in a `u64`. However, the likelihood that any single chess game achieves
//! a colission within its own game history is a statistical impossibility.
//!
//! The random values used in this library are generated using [`rand::rngs::SmallRng`]
//! seeded with a set seed of the first 32 bytes of the ASCII representation of π.

use std::{
    hash::{BuildHasher, Hasher},
    ops::BitXor,
    sync::LazyLock,
};

use chrono::format::Colons;
use rand::{Rng, RngCore, SeedableRng, rngs::SmallRng};
use static_init::Lazy;
use strum::VariantArray;

use crate::model::*;
use crate::{
    bitboard::{utils::bitor_sum, vision::PieceVision},
    biterate,
};

/// The rng state used to generate all the random values needed by the tables
/// in this module. Seeded with the bytes `3.141592653589793238462643383279`.
///
/// Discards the first 1000 values just in case.
pub fn pi_rng() -> SmallRng {
    let mut res = SmallRng::from_seed(*b"3.141592653589793238462643383279");
    for _ in 0..1000 {
        res.next_u64();
    }
    res
}

/// The Zobrist hash values of the [`crate::model::Transients`] information
/// of the chess position, as well as the hash of the currently active player being
/// black.
pub trait ZobristDetails {
    /// The hash of a given en-passant state, with 0 for en-passant not being possible. Must disambiguate transposed positions of pawn pushes.
    fn hash_en_passant(&self, ep: Option<EnPassant>) -> u64;
    /// Hash the sate of the castling rights, the argument is intended to be [`Transients.rights`](crate::model::Transients#structfield.rights).
    fn hash_rights(&self, rights: [[bool; 2]; 2]) -> u64;
    /// The value representinb black to move. If it is white to move, no extra information is added to the hash.
    fn black(&self) -> u64;
}

/// The default representation of the [`ZobristDetails`] trait, used for
/// inclusion-as-inheritance in the zobrist table implementations.
#[derive(Debug, Clone)]
pub struct DefaultZobristDetails {
    /// The file where en-passant capture is possible.
    pub ep_files: [u64; 8],
    /// Castling rights, indexed the same as [`Transients.rights`](crate::model::Transients#structfield.rights)
    pub rights: [[u64; 2]; 2],
    /// Value included in the hash when it is black to move.
    pub black_to_move: u64,
}

impl DefaultZobristDetails {
    /// Initialize the random values from a generator.
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
        for c in [ChessColor::WHITE, ChessColor::BLACK] {
            for d in [CastlingDirection::EAST, CastlingDirection::WEST] {
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

/// Delegation trait, allowing default implementation of [`ZobristDetails`]
trait HasDefaultZobristDetails {
    fn default_details(&self) -> &DefaultZobristDetails;
}

/// Delegating implementation of the [`ZobristDetails`] to the included
/// [`DefaultZobristDetails`] struct
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

/// Full zobrist hashing trait, also allowing for zobrist hashing of
/// the chessboard itself.
pub trait ZobristTables: ZobristDetails + 'static {
    /// Reference a statically allocated singleton instance.
    fn static_table() -> &'static Self;

    /// Hash a full 12-mask bitboard. See
    /// [`FullBitBoard`](crate::model::bitboard::FullBitBoard)
    /// for more information.
    fn hash_full_bitboard(&self, masks: &[[u64; 6]; 2]) -> u64;

    /// Hash a compact 8-mask bitboard. See
    /// [`CompactBitBoard`](crate::model::bitboard::CompactBitBoard)
    /// for more information.
    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64;

    /// Hash the relevant information of chess pieces moving around on
    /// a bitboard.
    ///
    /// - `bits` - this bit mask denotes the updated squares, and by the nature of
    /// Zobrist hashing it is irrelevant which squares the chessmen move to
    /// and from.
    fn hash_move(&self, player: ChessColor, man: ChessEchelon, bits: u64) -> u64;

    /// Hash a single square instead of a full mask.
    fn hash_square(&self, player: ChessColor, man: ChessEchelon, sq: Square) -> u64;

    /// Hash a castling move.
    fn hash_castling(&self, player: ChessColor, king_bits: u64, rook_bits: u64) -> u64;
}

/// Compact Zobrist hashing tables.
///
/// An implementation of zobrist hashing using less
/// space than the naive implementation, at the cost of requiring
/// twice as many XOR-operations to hash a chess move.
///
/// This is done by storing a separate value for 'black chessman' and
/// 'white chessman' for each square, along with one for each echelon
/// of chessman. The total number of values needed to has the chessboard
/// is thus only 512 (2 colors + 6 echelons, 64 squares.)
#[derive(Debug, Clone)]
pub struct CompactZobristTables {
    pub men: [[u64; 64]; 6],
    pub colors: [[u64; 64]; 2],
    pub details: DefaultZobristDetails,
}

impl CompactZobristTables {
    /// Initialize the random values from the [`pi_rng`] generator.
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

    /// Hash a mask of the chessmen of a particular color, regardless of echelons.
    #[inline]
    fn hash_color_mask(&self, color: ChessColor, mask: u64) -> u64 {
        let mut res = 0;
        biterate! {for sq in mask; {
            res ^= self.colors[color.ix()][sq as usize & 0x3F];
        }}
        res
    }

    /// Hash a mask of the chessmen of a given echelon regardless of color.
    #[inline]
    fn hash_man_mask(&self, man: ChessEchelon, mask: u64) -> u64 {
        let mut res = 0;
        biterate! {for sq in mask; {
            res ^= self.men[man.ix()][sq as usize & 0x3F];
        }}
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

    fn hash_move(&self, player: ChessColor, man: ChessEchelon, bits: u64) -> u64 {
        self.hash_color_mask(player, bits) ^ self.hash_man_mask(man, bits)
    }

    fn hash_square(&self, player: ChessColor, man: ChessEchelon, sq: Square) -> u64 {
        self.men[man.ix()][sq.ix()] ^ self.colors[player.ix()][sq.ix()]
    }

    fn hash_castling(&self, player: ChessColor, king_bits: u64, rook_bits: u64) -> u64 {
        self.hash_man_mask(ChessEchelon::KING, king_bits)
            ^ self.hash_man_mask(ChessEchelon::ROOK, rook_bits)
            ^ self.hash_color_mask(player, king_bits | rook_bits)
    }

    /// Hashing a full bitboard is less efficient in this implementation.
    fn hash_full_bitboard(&self, masks: &[[u64; 6]; 2]) -> u64 {
        let mut res = 0;
        res ^= self.hash_color_mask(ChessColor::WHITE, bitor_sum(&masks[ChessColor::WHITE.ix()]));
        res ^= self.hash_color_mask(ChessColor::BLACK, bitor_sum(&masks[ChessColor::BLACK.ix()]));

        for man in ChessEchelon::VARIANTS {
            res ^= self.hash_man_mask(
                *man,
                masks[ChessColor::WHITE.ix()][man.ix()] | masks[ChessColor::WHITE.ix()][man.ix()],
            );
        }

        res
    }

    /// Hashing a compact bitboard is more efficient in this implementation.
    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64 {
        let mut res = 0;
        for man in ChessEchelon::VARIANTS {
            res ^= self.hash_man_mask(*man, men[man.ix()]);
        }

        for color in [ChessColor::WHITE, ChessColor::BLACK] {
            res ^= self.hash_color_mask(color, colors[color.ix()]);
        }

        res
    }
}

/// The naive implementation of a Zobrist hashing table.
///
/// This uses 756 `u64`s to hash the board state (two players,
/// six echelons, 64 squares.)
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
    /// Initialize the random values from the [`pi_rng`] generator.
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

    /// Hash a mask of particular color and echelon of chess men.
    #[inline]
    fn hash_mask(&self, color: ChessColor, man: ChessEchelon, mut mask: u64) -> u64 {
        let mut res = 0;
        let board = self.masks[color.ix()][man.ix()];
        biterate! {for sq in mask; {
            res ^= board[sq.ix()];
        }}
        res
    }
}

static FULL_ZOBRIST: LazyLock<FullZobristTables> = LazyLock::new(FullZobristTables::new);

impl ZobristTables for FullZobristTables {
    fn static_table() -> &'static Self {
        &FULL_ZOBRIST
    }

    #[inline]
    fn hash_move(&self, player: ChessColor, man: ChessEchelon, bits: u64) -> u64 {
        self.hash_mask(player, man, bits)
    }

    #[inline]
    fn hash_square(&self, player: ChessColor, man: ChessEchelon, sq: Square) -> u64 {
        self.masks[player.ix()][man.ix()][sq.ix()]
    }

    #[inline]
    fn hash_castling(&self, player: ChessColor, king_bits: u64, rook_bits: u64) -> u64 {
        self.hash_mask(player, ChessEchelon::KING, king_bits)
            ^ self.hash_mask(player, ChessEchelon::ROOK, rook_bits)
    }

    /// Hashing a full bitboard is more efficient in this implementation.
    #[inline]
    fn hash_full_bitboard(&self, masks: &[[u64; 6]; 2]) -> u64 {
        let mut res = 0;
        for c in [ChessColor::WHITE, ChessColor::BLACK] {
            for m in ChessEchelon::VARIANTS {
                res ^= self.hash_mask(c, *m, masks[c.ix()][m.ix()]);
            }
        }
        res
    }

    /// Hashing a compact bitboard is less efficient in this implementation.
    #[inline]
    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64 {
        let mut res = 0;
        for c in [ChessColor::WHITE, ChessColor::BLACK] {
            for m in ChessEchelon::VARIANTS {
                res ^= self.hash_mask(c, *m, colors[c.ix()] & men[m.ix()]);
            }
        }
        res
    }
}

/// Dummy implementation of Zobrist hashing.
///
/// No hashing is done, 100% colission rate.
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

    fn hash_full_bitboard(&self, masks: &[[u64; 6]; 2]) -> u64 {
        0
    }

    fn hash_compact(&self, colors: &[u64; 2], men: &[u64; 6]) -> u64 {
        0
    }

    fn hash_move(&self, player: ChessColor, man: ChessEchelon, bits: u64) -> u64 {
        0
    }

    fn hash_square(&self, player: ChessColor, man: ChessEchelon, sq: Square) -> u64 {
        0
    }

    fn hash_castling(&self, player: ChessColor, king_bits: u64, rook_bits: u64) -> u64 {
        0
    }
}

/// An implementation of the [`std::hash::Hasher`] trait for Zobrist hashing.
///
/// Used with types from `std` collections, when using zobrist hashing in some
/// way. Essentially just identity hashing. Only works for `u64`s, ignores all
/// other values.
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

impl BuildHasher for ZobHasher {
    type Hasher = ZobHasher;

    fn build_hasher(&self) -> Self::Hasher {
        ZobHasher(0)
    }
}
