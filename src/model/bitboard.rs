//! # The bitboard representation of the chessboard.
//!
//! This is the currently accepted most efficient computational
//! representation of a chessboard, which is used by many top
//! chess engines.
//!
//! Bitboards function on the observation that there are 64 squares
//! on a chessboard, and 64 bits in a `u64`. Hence one bit can be used
//! to represent the absence or presence of some quantity on a chessboard
//! square.
//!
//! Utilizing this observation, we use a separate `u64` for each kind
//! of chessman. One denoting the positions of all the white pawns, one
//! denoting the black pawns, and so on.
//!
//! This allows very fast lookups of the presence or absence of pieces,
//! as well as several advanced arithmetic tricks to compute difficult
//! quantities.
//!
//! Three distinct implementations are provided in this module, for
//! profiling. Their interfaces are identical and they can be substituted
//! for one another without loss of correctness.

use crate::model::{
    ChessColor, ChessCommoner, ChessEchelon, ChessMan, EnPassant, Square, Transients,
    castling::{CLASSIC_CASTLING, Castling},
    hash::ZobristTables,
    utils::{SliceExtensions, bitor_sum},
};
use strum::VariantArray;

/// The basic operations of a bitboard.
pub trait BitBoard: ChessBoard {
    /// Move a chessman by applying an XOR-operation to the
    /// bitboard corresponding the chessmen of that echelon and color.
    /// (The mask usually has only two bits set.)
    ///
    /// Notably this operation, because it uses XOR, is an involution.
    fn xor(&mut self, color: ChessColor, ech: ChessEchelon, mask: u64);

    /// Retrieve the bitboard representing a given echelon and color of chessman.
    fn men(&self, color: ChessColor, ech: ChessEchelon) -> u64;

    /// Determine if a chessman of some echelon stands on a square
    fn ech_at(&self, sq: Square) -> Option<ChessEchelon>;

    /// Determine if a chessman of some non-king echelon stands on a square
    fn comm_at(&self, sq: Square) -> Option<ChessCommoner> {
        self.ech_at(sq).and_then(ChessCommoner::from_echelon)
    }

    /// Retrieve the bitboard represeting the squares occuipied by all the chessmen of one color.
    fn color(&self, color: ChessColor) -> u64;

    /// Retrieve the bitboard representing all occupied squares.
    fn total(&self) -> u64;
}

/// A proper chessboard.
pub trait ChessBoard: MetaBoard {
    /// The classic chess start position.
    ///
    /// ```text
    /// ♜♞♝♛♚♝♞♜
    /// ♟♟♟♟♟♟♟♟
    ///
    /// ♙♙♙♙♙♙♙♙
    /// ♖♘♗♕♔♗♘♖
    /// ```
    ///
    /// (The above figure only looks correct when viewed as dark text on light background.
    /// The unicode characters for chessmen do not account for dark mode.)
    fn startpos<ZT: ZobristTables>() -> Self;

    /// Optional sanity checking.
    ///
    /// Bitboards, unlike mailboxes, do not have
    /// an inherent protection against chessmen
    /// occupying the same squares, and so this
    /// function exists to allow debugging.
    fn sanity_check<ZT: ZobristTables>(&self);

    /// Recompute the Zobrist hash of this table.
    fn rehash<ZT: ZobristTables>(&self) -> u64;
}

/// The metadata associated with a chessboard.
///
/// Includes:
///
/// - Active player color and turn number
/// - Zobrist hash of the current position (see [`hash`](crate::model::hash))
/// - The transient game state information
pub trait MetaBoard {
    /// The current state of the transient information.
    ///
    /// Yes, [`Transients`] includes a field named `rights` 🏳️‍⚧️
    ///
    /// I am very funny.
    fn trans(&self) -> Transients;

    /// Update the half-move clock transient values.
    fn set_halfmove_clock(&mut self, val: u8);

    /// Update the castling rights in the transient values.
    fn set_castling_rights(&mut self, rights: [[bool; 2]; 2]);

    /// Update the _en-passant_ information in the transient values.
    fn set_en_passant(&mut self, eps: Option<EnPassant>);

    /// Current Zobrist hash of the position.
    fn curr_hash(&self) -> u64;

    /// Update the Zobrist hash with a given delta hash.
    fn hash(&mut self, hash: u64);

    /// Current active player color and turn number.
    ///
    /// In game theory, a 'ply' is the technical term for
    /// a player making a single move. In chess, each turn
    /// includes both a move from white and a move from black,
    /// hence a ply in a chess game can be uniquely denoted by
    /// the turn number and the active player.
    fn ply(&self) -> (ChessColor, u16);

    /// Increment the ply, i.e. swap active player color and increment
    /// the turn counter if the swap was black-to-white.
    fn next_ply(&mut self);

    /// Decrement the ply, i.e. swap active player color and decrement
    /// the turn counter if the swap was white-to-black.
    ///
    /// Trying to decrement the ply below the starting value of (1, white)
    /// is unspecified behavior.
    fn prev_ply(&mut self);

    /// The metadata associated with castling rules for the current
    /// game.
    ///
    /// The chess variants Chess960 and Chess480 have different castling
    /// rules, and so castling rules are specified in data, rather than
    /// hard-coded.
    fn castling(&self) -> &'static Castling;
}

/// Default implementation of the [`MetaBoard`] trait, used for
/// inclusion-as-inheritance in the bitboard implementations.
#[derive(Debug, Clone, Copy)]
pub struct DefaultMetaBoard {
    pub castling: &'static Castling,
    pub hash: u64,
    pub turn: u16,
    pub player: ChessColor,
    pub trans: Transients,
}

/// Equality comparison that ignores the turn counter and the
/// [`Transients.halfmove_clock`](crate::model::Transients#structfield.halfmove_clock),
/// for the purposes of comparing the equivalence of board positions atemporally.
impl PartialEq for DefaultMetaBoard {
    fn eq(&self, other: &Self) -> bool {
        self.player == other.player
            && self.trans.en_passant == other.trans.en_passant
            && self.trans.rights == other.trans.rights
    }
}

impl MetaBoard for DefaultMetaBoard {
    #[inline]
    fn castling(&self) -> &'static Castling {
        self.castling
    }

    #[inline]
    fn trans(&self) -> Transients {
        self.trans
    }

    #[inline]
    fn ply(&self) -> (ChessColor, u16) {
        (self.player, self.turn)
    }

    #[inline]
    fn next_ply(&mut self) {
        self.player = self.player.opp();
        if self.player == ChessColor::WHITE {
            self.turn += 1;
        }
    }

    #[inline]
    fn prev_ply(&mut self) {
        if self.player == ChessColor::WHITE {
            self.turn -= 1;
        }
        self.player = self.player.opp();
    }

    #[inline]
    fn curr_hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    fn hash(&mut self, hash: u64) {
        self.hash ^= hash;
    }

    #[inline]
    fn set_halfmove_clock(&mut self, val: u8) {
        self.trans.halfmove_clock = val;
    }

    #[inline]
    fn set_castling_rights(&mut self, rights: [[bool; 2]; 2]) {
        self.trans.rights = rights;
    }

    #[inline]
    fn set_en_passant(&mut self, rights: Option<EnPassant>) {
        self.trans.en_passant = rights;
    }
}

impl ChessBoard for DefaultMetaBoard {
    /// Starting position sans the actual chessmen.
    ///
    /// White to move on turn 1, castling allowed
    /// in both directtions for both players.
    fn startpos<ZT: ZobristTables>() -> Self {
        let mut res = Self {
            castling: &CLASSIC_CASTLING,
            hash: 0,
            player: ChessColor::WHITE,
            turn: 1,
            trans: Transients {
                en_passant: None,
                halfmove_clock: 0,
                rights: [[true; 2]; 2],
            },
        };
        res.hash = res.rehash::<ZT>();
        res
    }

    /// Performs the following checks:
    ///
    /// - The procedurally computed hash is equal to the recomputed hash.
    fn sanity_check<ZT: ZobristTables>(&self) {
        assert_eq!(self.curr_hash(), self.rehash::<ZT>());
    }

    #[inline]
    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        let zobristtable = ZT::static_table();
        zobristtable.black()
            ^ zobristtable.hash_rights(self.trans.rights)
            ^ zobristtable.hash_en_passant(self.trans.en_passant)
    }
}

/// Delegation trait, allowing default implementation of [`MetaBoard`]
pub trait HasDefaultMetaBoard {
    fn metaboard(&self) -> &DefaultMetaBoard;
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard;
}

impl<BB: HasDefaultMetaBoard + Clone> MetaBoard for BB {
    #[inline]
    fn trans(&self) -> Transients {
        self.metaboard().trans()
    }

    #[inline]
    fn curr_hash(&self) -> u64 {
        self.metaboard().curr_hash()
    }

    #[inline]
    fn hash(&mut self, hash: u64) {
        self.metaboard_mut().hash(hash)
    }

    #[inline]
    fn ply(&self) -> (ChessColor, u16) {
        self.metaboard().ply()
    }

    #[inline]
    fn next_ply(&mut self) {
        self.metaboard_mut().next_ply();
    }

    #[inline]
    fn prev_ply(&mut self) {
        self.metaboard_mut().prev_ply();
    }

    #[inline]
    fn castling(&self) -> &'static Castling {
        self.metaboard().castling()
    }

    #[inline]
    fn set_halfmove_clock(&mut self, val: u8) {
        self.metaboard_mut().set_halfmove_clock(val);
    }

    #[inline]
    fn set_castling_rights(&mut self, rights: [[bool; 2]; 2]) {
        self.metaboard_mut().set_castling_rights(rights);
    }

    #[inline]
    fn set_en_passant(&mut self, eps: Option<EnPassant>) {
        self.metaboard_mut().set_en_passant(eps);
    }
}

/// The compact bitboard representation.
///
/// This representation uses a total of 8 `u64` values to represent
/// the state of the board, one for each color, and one for each echelon.
///
/// The particular bitboard representing a given type of chessmen is
/// then obtained as the binary AND of the color mask and the echelon mask.
///
/// Care must be taken when updating to move both the piece and the color.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompactBitBoard {
    pub ech: [u64; 6],
    pub colors: [u64; 2],
    pub meta: DefaultMetaBoard,
}

impl BitBoard for CompactBitBoard {
    /// Updates both the echelon mask and the color mask separately.
    #[inline]
    fn xor(&mut self, color: ChessColor, ech: ChessEchelon, mask: u64) {
        self.ech[ech.ix()] ^= mask;
        self.colors[color.ix()] ^= mask;
    }

    /// Computed with a binary OR operation
    fn men(&self, color: ChessColor, ech: ChessEchelon) -> u64 {
        self.ech[ech.ix()] & self.colors[color.ix()]
    }

    /// Very efficiently directly on hand in this implementation
    fn color(&self, color: ChessColor) -> u64 {
        self.colors[color.ix()]
    }

    /// Very efficiently computed in a single binary OR operation
    fn total(&self) -> u64 {
        self.colors[ChessColor::WHITE.ix()] | self.colors[ChessColor::BLACK.ix()]
    }

    /// Very efficiently computed as there are only 6 masks to check
    fn ech_at(&self, sq: Square) -> Option<ChessEchelon> {
        let bit = 1 << sq.ix();
        for c in ChessEchelon::VARIANTS.clones() {
            if (self.ech[c.ix()] & bit) != 0 {
                return Some(c);
            }
        }
        None
    }

    /// Very efficiently computed as there are only 6 masks to check
    fn comm_at(&self, sq: Square) -> Option<ChessCommoner> {
        let bit = 1 << sq.ix();
        for c in ChessCommoner::VARIANTS.clones() {
            if (self.ech[c.ix()] & bit) != 0 {
                return Some(c);
            }
        }
        None
    }
}

impl HasDefaultMetaBoard for CompactBitBoard {
    #[inline]
    fn metaboard(&self) -> &DefaultMetaBoard {
        &self.meta
    }

    #[inline]
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard {
        &mut self.meta
    }
}

impl ChessBoard for CompactBitBoard {
    fn startpos<ZT: ZobristTables>() -> Self {
        let mut res = Self {
            ech: [
                0x00FF_0000_0000_FF00,
                0x4200_0000_0000_0042,
                0x2400_0000_0000_0024,
                0x8100_0000_0000_0081,
                0x0800_0000_0000_0008,
                0x1000_0000_0000_0010,
            ],
            colors: [0x0000_0000_0000_FFFF, 0xFFFF_0000_0000_0000],
            meta: DefaultMetaBoard::startpos::<ZT>(),
        };
        res.meta.hash = res.rehash::<ZT>();
        res
    }

    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        self.metaboard().rehash::<ZT>() ^ ZT::static_table().hash_compact(&self.colors, &self.ech)
    }

    /// Performs the following checks:
    ///
    /// - Are the echelon masks non-overlapping?
    /// - Are the color masks non-overlapping?
    /// - Are the sum of the echelon masks equal to the sum of the color masks?
    /// - Is the procedurally updated hash equal to the recomputed hash?
    fn sanity_check<ZT: ZobristTables>(&self) {
        for p1 in ChessEchelon::VARIANTS {
            for p2 in ChessEchelon::VARIANTS {
                let (p1, p2) = (*p1, *p2);
                if p1 >= p2 {
                    continue;
                }

                assert_eq!(
                    self.ech[p1.ix()] & self.ech[p2.ix()],
                    0,
                    "{:?} and {:?} overlap",
                    p1,
                    p2
                );
            }
        }

        assert_eq!(
            self.colors[ChessColor::WHITE.ix()] | self.colors[ChessColor::BLACK.ix()],
            0,
            "white and black overlap",
        );

        let mut white = 0;
        let mut black = 0;
        let mut total = 0;

        for p in &self.ech {
            let p = *p;
            white |= self.colors[ChessColor::WHITE.ix()] & p;
            black |= self.colors[ChessColor::BLACK.ix()] & p;
            total |= p;
        }

        assert_eq!(
            white,
            self.colors[ChessColor::WHITE.ix()],
            "sum of white-masked pieces not equal to white"
        );

        assert_eq!(
            black,
            self.colors[ChessColor::BLACK as usize],
            "sum of black-masked pieces not equal to black"
        );

        assert_eq!(
            total,
            white | black,
            "sum of pieces not equal to disjunction of colors"
        );

        assert_eq!(
            self.curr_hash(),
            self.rehash::<ZT>(),
            "procedural hash mismatch"
        );
    }
}

/// The naive bitboard representation.
///
/// This representation uses a total of 12 `u64` values to represent
/// the state of the board, one for each kind of chessman.
#[derive(Debug, Clone)]
pub struct FullBitBoard {
    masks: [[u64; 6]; 2],
    meta: DefaultMetaBoard,
}

impl BitBoard for FullBitBoard {
    /// Computed in a single XOR operation.
    #[inline]
    fn xor(&mut self, color: ChessColor, ech: ChessEchelon, mask: u64) {
        self.masks[color.ix()][ech.ix()] ^= mask;
    }

    /// Directly on hand.
    #[inline]
    fn men(&self, color: ChessColor, ech: ChessEchelon) -> u64 {
        self.masks[color.ix()][ech.ix()]
    }

    /// Computed as the sum of all the chessmen masks of one color.
    #[inline]
    fn color(&self, color: ChessColor) -> u64 {
        bitor_sum(&self.masks[color.ix()])
    }

    /// Sum of all bitboards in this representation.
    #[inline]
    fn total(&self) -> u64 {
        self.color(ChessColor::WHITE) | self.color(ChessColor::BLACK)
    }

    /// Not so efficiently computed as there are twelve masks to check
    fn ech_at(&self, sq: Square) -> Option<ChessEchelon> {
        let bit = 1 << sq.ix();
        for c in ChessEchelon::VARIANTS.clones() {
            if (self.masks[ChessColor::WHITE.ix()][c.ix()] & bit) != 0
                || (self.masks[ChessColor::BLACK.ix()][c.ix()] & bit) != 0
            {
                return Some(c);
            }
        }
        None
    }

    /// Not so efficiently computed as there are ten masks to check
    fn comm_at(&self, sq: Square) -> Option<ChessCommoner> {
        let bit = 1 << sq.ix();
        for c in ChessCommoner::VARIANTS.clones() {
            if (self.masks[ChessColor::WHITE.ix()][c.ix()] & bit) != 0
                || (self.masks[ChessColor::BLACK.ix()][c.ix()] & bit) != 0
            {
                return Some(c);
            }
        }
        None
    }
}

impl HasDefaultMetaBoard for FullBitBoard {
    #[inline]
    fn metaboard(&self) -> &DefaultMetaBoard {
        &self.meta
    }

    #[inline]
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard {
        &mut self.meta
    }
}

impl ChessBoard for FullBitBoard {
    fn startpos<ZT: ZobristTables>() -> Self {
        let white = [0xFF00, 0x42, 0x24, 0x81, 0x08, 0x10u64];
        let black = white.map(|m| m.swap_bytes());
        let mut res = Self {
            masks: [white, black],
            meta: DefaultMetaBoard::startpos::<ZT>(),
        };
        res.meta.hash = res.rehash::<ZT>();
        res
    }

    /// Performs the following checks:
    ///
    /// - All the bit masks are non-overlapping
    /// - The procedurally computed hash is equal to the recomputed hash
    fn sanity_check<ZT: ZobristTables>(&self) {
        for p1 in ChessEchelon::VARIANTS {
            for p2 in ChessEchelon::VARIANTS {
                for c1 in [ChessColor::WHITE, ChessColor::BLACK] {
                    for c2 in [ChessColor::WHITE, ChessColor::BLACK] {
                        let (p1, p2) = (*p1, *p2);
                        if (p1, c1) >= (p2, c2) {
                            continue;
                        }

                        assert_eq!(
                            self.masks[c1.ix()][p1.ix()] & self.masks[c2.ix()][p2.ix()],
                            0,
                            "{:?} {:?} and {:?} {:?} overlap",
                            c1,
                            p1,
                            c2,
                            p2
                        );
                    }
                }
            }
        }

        assert_eq!(self.metaboard().curr_hash(), self.rehash::<ZT>());
    }

    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        self.metaboard().rehash::<ZT>() ^ ZT::static_table().hash_full_bitboard(&self.masks)
    }
}

/// A slight optimization of the [`FullBitBoard`].
///
/// In this version, each side of the chessboard is updated
/// concurrently with the individual chessmen masks, a kind of
/// compromise between the compact and naive implementation.
///
/// This is done because move generation relies heavily on being
/// able to compute the occupancies of colors and the whole board
/// for determining which squares are blocked.
#[derive(Debug, Clone)]
pub struct FullerBitBoard {
    pub bitboard: FullBitBoard,
    pub total: [u64; 2],
}

impl BitBoard for FullerBitBoard {
    /// Updates both the chessman mask and the color mask separately.
    #[inline]
    fn xor(&mut self, color: ChessColor, ech: ChessEchelon, mask: u64) {
        self.bitboard.xor(color, ech, mask);
        self.total[color.ix()] ^= mask;
    }

    /// On hand directly.
    #[inline]
    fn men(&self, color: ChessColor, ech: ChessEchelon) -> u64 {
        self.bitboard.men(color, ech)
    }

    /// On hand directly.
    #[inline]
    fn color(&self, color: ChessColor) -> u64 {
        self.total[color.ix()]
    }

    /// Computed in a single OR-instruction.
    #[inline]
    fn total(&self) -> u64 {
        bitor_sum(&self.total)
    }

    /// Delegated
    #[inline]
    fn ech_at(&self, sq: Square) -> Option<ChessEchelon> {
        self.bitboard.ech_at(sq)
    }

    /// Delegated
    #[inline]
    fn comm_at(&self, sq: Square) -> Option<ChessCommoner> {
        self.bitboard.comm_at(sq)
    }
}

impl HasDefaultMetaBoard for FullerBitBoard {
    #[inline]
    fn metaboard(&self) -> &DefaultMetaBoard {
        self.bitboard.metaboard()
    }

    #[inline]
    fn metaboard_mut(&mut self) -> &mut DefaultMetaBoard {
        self.bitboard.metaboard_mut()
    }
}

impl ChessBoard for FullerBitBoard {
    fn startpos<ZT: ZobristTables>() -> Self {
        Self {
            bitboard: FullBitBoard::startpos::<ZT>(),
            total: [0x0000_0000_0000_FF00, 0x00FF_0000_0000_0000],
        }
    }

    fn sanity_check<ZT: ZobristTables>(&self) {
        self.bitboard.sanity_check::<ZT>();
        assert_eq!(
            self.total[ChessColor::WHITE.ix()],
            self.bitboard.color(ChessColor::WHITE),
            "white total is not sum of white pieces"
        );
        assert_eq!(
            self.total[ChessColor::BLACK.ix()],
            self.bitboard.color(ChessColor::BLACK),
            "black total is not sum of black pieces"
        )
    }

    #[inline]
    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        self.bitboard.rehash::<ZT>()
    }
}
