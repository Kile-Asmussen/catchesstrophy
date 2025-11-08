use strum::{EnumIs, EnumIter, VariantArray, VariantNames};

/// Representation of the squares on a chessboard.
///
/// This enum uses the convention of numbering
/// squares starting with a1 = 0 and then counting
/// up over the files first, b1 = 1, c1 = 2, ... and then the
/// ranks, a2 = 8, a3 = 16, ... ending with h8 = 63.
///
/// This is the so called file-major little-endian layout.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
     VariantNames, EnumIter)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Square {
    a1 = 0o00, b1 = 0o01, c1 = 0o02, d1 = 0o03, e1 = 0o04, f1 = 0o05, g1 = 0o06, h1 = 0o07,
    a2 = 0o10, b2 = 0o11, c2 = 0o12, d2 = 0o13, e2 = 0o14, f2 = 0o15, g2 = 0o16, h2 = 0o17,
    a3 = 0o20, b3 = 0o21, c3 = 0o22, d3 = 0o23, e3 = 0o24, f3 = 0o25, g3 = 0o26, h3 = 0o27,
    a4 = 0o30, b4 = 0o31, c4 = 0o32, d4 = 0o33, e4 = 0o34, f4 = 0o35, g4 = 0o36, h4 = 0o37,
    a5 = 0o40, b5 = 0o41, c5 = 0o42, d5 = 0o43, e5 = 0o44, f5 = 0o45, g5 = 0o46, h5 = 0o47,
    a6 = 0o50, b6 = 0o51, c6 = 0o52, d6 = 0o53, e6 = 0o54, f6 = 0o55, g6 = 0o56, h6 = 0o57,
    a7 = 0o60, b7 = 0o61, c7 = 0o62, d7 = 0o63, e7 = 0o64, f7 = 0o65, g7 = 0o66, h7 = 0o67,
    a8 = 0o70, b8 = 0o71, c8 = 0o72, d8 = 0o73, e8 = 0o74, f8 = 0o75, g8 = 0o76, h8 = 0o77,
}

impl Square {
    /// Use this Square as an array index.
    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }

    /// Infallible conversion from a u8 by way of truncating the
    /// extraneous bits.
    #[inline]
    pub fn from_u8(ix: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(ix & 0x3Fu8) }
    }

    /// Split a square into file and rank
    #[inline]
    pub fn coords(self) -> (BoardFile, BoardRank) {
        (
            BoardFile::from_u8(self as u8),
            BoardRank::from_u8((self as u8 & 0x38) >> 3),
        )
    }

    /// Split a square into file and rank
    #[inline]
    pub fn from_coords(f: BoardFile, r: BoardRank) -> Self {
        Self::from_u8(f as u8 | (r as u8) << 3)
    }

    /// Mirror chessboard north to south
    #[inline]
    pub fn mirror_ns(self) -> Self {
        Self::from_u8(self as u8 ^ 0x38u8)
    }

    /// Mirror chessboard east to west
    #[inline]
    pub fn mirror_ew(self) -> Self {
        Self::from_u8(self as u8 ^ 0x7u8)
    }

    /// Rotate chessboard 180 degrees
    #[inline]
    pub fn rotate(self) -> Self {
        Self::from_u8(63u8 - self as u8)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum BoardRank {
    _1 = 0,
    _2 = 1,
    _3 = 2,
    _4 = 3,
    _5 = 4,
    _6 = 5,
    _7 = 6,
    _8 = 7,
}

impl BoardRank {
    pub const VARIANTS: &'static [&'static str] = &["1", "2", "3", "4", "5", "6", "7", "8"];

    /// Use this rank as an array index.
    #[inline]
    pub fn ix(self) -> usize {
        (self as usize) << 3
    }

    /// Infallible conversion from a u8 by way of truncating the
    /// extraneous bits.
    #[inline]
    pub fn from_u8(ix: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(ix & 0x7) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum BoardFile {
    a_ = 0,
    b_ = 1,
    c_ = 2,
    d_ = 3,
    e_ = 4,
    f_ = 5,
    g_ = 6,
    h_ = 7,
}

impl BoardFile {
    pub const VARIANTS: &'static [&'static str] = &["a", "b", "c", "d", "e", "f", "g", "h"];

    /// Use this file as an array index.
    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }

    /// Infallible conversion from a u8 by way of truncating the
    /// extraneous bits.
    #[inline]
    pub fn from_u8(ix: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(ix & 0x7) }
    }
}

/// Representation of a chessman.
///
/// The discriminants allows niche optimization with a byte value of
/// 0 representing absence, and with the sign representing color.
///
/// The name chessman is of British-English origin, and though archaic
/// is used because it allows a distinction between pawns and pieces.
/// Using pieces to also refer to pawns carries ambiguity.
///
/// Despite the name, the queens are still fierce... well, queens, full of
/// girl power!
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray, Hash)]
#[repr(i8)]
pub enum ChessMan {
    BLACK_KING = -6,
    BLACK_QUEEN = -5,
    BLACK_ROOK = -4,
    BLACK_BISHOP = -3,
    BLACK_KNIGHT = -2,
    BLACK_PAWN = -1,
    WHITE_PAWN = 1,
    WHITE_KNIGHT = 2,
    WHITE_BISHOP = 3,
    WHITE_ROOK = 4,
    WHITE_QUEEN = 5,
    WHITE_KING = 6,
}

/// Representation of color of a player or chessman.
///
/// The choice here to not to mirror the convention of black = `-1` and
/// white = `1` as used in the [`ChessMan`] enum is because this is used
/// extensively in indexing of arrays of the form `[<white value>, <black value>]`.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIs)]
#[repr(u8)]
pub enum ChessColor {
    WHITE = 0,
    BLACK = 1,
}

impl ChessColor {
    /// Opposing color.
    #[inline]
    pub fn opp(self) -> Self {
        unsafe { std::mem::transmute(self as u8 ^ 1) }
    }

    /// Sign value of associated chessman color.
    #[inline]
    pub fn sign(self) -> i8 {
        match self {
            Self::WHITE => 1,
            Self::BLACK => -1,
        }
    }

    /// Associated array index.
    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }
}

/// Extracting the color of a chessman.
impl From<ChessMan> for ChessColor {
    fn from(value: ChessMan) -> Self {
        if (value as i8) < 0 {
            Self::BLACK
        } else {
            Self::WHITE
        }
    }
}

/// Representation of the piece typs of chessmen.
///
/// The discriminant values of this enum are the absolute
/// values of the [`ChessMan`] enum, or equivalently, the white chessmen.
///
/// This enum is used _far_ more extensively than
/// its parent enum, on account of most of the implementation
/// relying on arrays of length six to represent information about
/// each rank of chessmen.
///
/// This enum is further subdivided into named ranges.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, VariantArray)]
#[repr(u8)]
pub enum ChessPiece {
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6,
}

impl ChessPiece {
    /// Use as an array index: equal to one less than the discriminant value.
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

/// Extracting the rank of a chessman.
impl From<ChessMan> for ChessPiece {
    #[inline]
    fn from(value: ChessMan) -> Self {
        unsafe { std::mem::transmute((value as i8).abs() as u8) }
    }
}

/// Subset inclusion.
impl From<ChessOfficer> for ChessPiece {
    #[inline]
    fn from(value: ChessOfficer) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

/// Subset inclusion.
impl From<ChessPawn> for ChessPiece {
    #[inline]
    fn from(value: ChessPawn) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

/// Subset inclusion.
impl From<PawnPromotion> for ChessPiece {
    #[inline]
    fn from(value: PawnPromotion) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

/// Subset inclusion.
impl From<ChessCommoner> for ChessPiece {
    #[inline]
    fn from(value: ChessCommoner) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

/// Representation of the chess pawn, i.e. not an officer.
///
/// Mostly included for completeness' sake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ChessPawn {
    PAWN = 1,
}

impl ChessPawn {
    /// See [`ChessPiece::ix`].
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

/// Representation of the chess officers, that is, not pawns.
///
/// In several instances in this codebase, the exclusion of pawns
/// at a type-level is a convenient guarantee.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ChessOfficer {
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
    KING = 6,
}

impl ChessOfficer {
    /// See [`ChessPiece::ix`].
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

/// Representation of the chess commoners, that is, not kings.
///
/// In several instances in this codebase, the exclusion of kings
/// at a type-level is a convenient guarantee.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, VariantArray, Hash)]
#[repr(u8)]
pub enum ChessCommoner {
    PAWN = 1,
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
}

impl ChessCommoner {
    /// See [`ChessPiece::ix`].
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }

    #[inline]
    pub fn from_piece(ech: ChessPiece) -> Option<Self> {
        if ech == ChessPiece::KING {
            None
        } else {
            unsafe { std::mem::transmute(ech as u8) }
        }
    }
}

/// Representation of the chess promotion echelons, that is, not pawns or kings.
///
/// In several instances in this codebase, the exclusion of pawns and kings
/// at a type-level is a convenient guarantee.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PawnPromotion {
    KNIGHT = 2,
    BISHOP = 3,
    ROOK = 4,
    QUEEN = 5,
}

impl PawnPromotion {
    /// See [`ChessPiece::ix`].
    #[inline]
    pub fn ix(self) -> usize {
        self as usize - 1
    }
}

/// Representation of the directions on a chessboard.
///
/// ```text
///  NE     North    NW
///      +7  +8  +9
/// East -1  ..  +1 West
///      -9  -8  -7
///  SE     south    SW
/// ```
///
/// This is the classic compass rose associated with the
/// '64'-representation of chessboard squares. For a given
/// square index, so long as it would not move off the board,
/// adding a direction value to it will result in the square
/// index in that direction.
///
/// Equivalently shifting a `u64` by the enum discriminant value,
/// with positive being a left shift and negative being a right shift,
/// the bits are moved on the chessboard (though one must mask out the
/// rollover files when shifting in directiosn other than north/south.)
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum CompassRose {
    NORTH = 8,
    WEST = 1,
    EAST = -1,
    SOUTH = -8,

    NORTHWEST = Self::NORTH as i8 + Self::WEST as i8,
    NORTHEAST = Self::NORTH as i8 + Self::EAST as i8,
    SOUTHWEST = Self::SOUTH as i8 + Self::WEST as i8,
    SOUTHEAST = Self::SOUTH as i8 + Self::EAST as i8,
}

/// Representation of the directions of castling.
///
/// Note here that the discriminant values are not equal
/// to the associated with [`CompassRose`], this is again
/// owing to their use as array indexes.
///
/// The naming convention is chosen to account for Chess960
/// and Chess480, wherein the rook's relative position to the
/// king is not fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CastlingDirection {
    /// Aka. the 'long' or 'queen-side' castling.
    EAST = 0,
    /// Aka. the 'short' or 'king-side' castling.
    WEST = 1,
}

impl CastlingDirection {
    /// Use as an array index.
    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }
}

/// Subset inclusion (with mapping.)
impl From<CastlingDirection> for CompassRose {
    #[inline]
    fn from(value: CastlingDirection) -> Self {
        match value {
            CastlingDirection::EAST => Self::EAST,
            CastlingDirection::WEST => Self::WEST,
        }
    }
}

/// Representations of the three special moves available in chess:
///
/// - Castling
/// - En-passant vulnerability and capture
/// - Pawn promotion
///
/// In particular the [`ChessCommoner`] maps directly into this enum.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum SpecialMove {
    PAWN = 1,   // Double push or en-passant capture
    KNIGHT = 2, // Promote to knight
    BISHOP = 3, // Promote to bishop
    ROOK = 4,   // Promote to rook
    QUEEN = 5,  // Promote to queen
    EAST = 6,   // Castling east
    WEST = 7,   // Castling west
}

/// Subset inclusion.
impl From<ChessPawn> for SpecialMove {
    #[inline]
    fn from(value: ChessPawn) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

/// Subset inclusion.
impl From<PawnPromotion> for SpecialMove {
    #[inline]
    fn from(value: PawnPromotion) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

/// Subset inclusion (with mapping.)
impl From<CastlingDirection> for SpecialMove {
    #[inline]
    fn from(value: CastlingDirection) -> Self {
        unsafe { std::mem::transmute(value as u8 + SpecialMove::EAST as u8) }
    }
}

/// Subset inclusion.
impl From<ChessCommoner> for SpecialMove {
    #[inline]
    fn from(value: ChessCommoner) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
}

impl ChessPawn {
    /// Attempt to convert from special move.
    pub fn from_special(special: Option<SpecialMove>) -> Option<Self> {
        if special == Some(SpecialMove::PAWN) {
            Some(ChessPawn::PAWN)
        } else {
            None
        }
    }
}

impl PawnPromotion {
    /// Attempt to convert from special move.
    pub fn from_special(special: Option<SpecialMove>) -> Option<Self> {
        let special = special?;
        if SpecialMove::KNIGHT <= special && special <= SpecialMove::QUEEN {
            Some(unsafe { std::mem::transmute(special) })
        } else {
            None
        }
    }
}

impl CastlingDirection {
    /// Attempt to convert from special move.
    pub fn from_special(special: Option<SpecialMove>) -> Option<Self> {
        let special = special?;
        if SpecialMove::EAST <= special {
            Some(unsafe { std::mem::transmute(special as u8 - SpecialMove::EAST as u8) })
        } else {
            None
        }
    }
}

/// Wrapper for potential moves that have not yet been verified legal,
/// that is they might put the moving player's king in check, or let
/// it remain in check.
///
/// Provided as syntactic salt for the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PseudoLegal(pub ChessMove);

/// Wrapper for moves that have not yet been verified legal, that is
/// they do not result in the moving player's king being in check
/// after the move is made.
///
/// Provided as syntactic salt for the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct LegalMove(pub ChessMove);

/// Representation of a move on a chessboard.
///
/// This is a 'fat' representation, rather than the 'compact'
/// representaiton that can fit in as little as 16-bits, and
/// has been chosen for ease of use on an API level, and potentially
/// increased compiler optimizations.
///
/// The moves are generally assumed to be produced by a pseudo-legal
/// move enumeration algorithm referencing a chessboard position. Attempting
/// to execute a move that is 'invalid' in a given chess position will
/// result in unspecified behavior --- that is, the only guarantee is soundness
/// within the rust semantics, not the rules of chess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChessMove {
    pub ech: ChessPiece,
    pub from: Square,
    pub to: Square,
    pub special: Option<SpecialMove>,
    pub capture: Option<ChessCommoner>,
}

impl ChessMove {
    /// Sanity check for enumerated moves for standard chess.
    ///
    /// Checks the following:
    ///
    /// - The start and end squares are different.
    /// - A castling move is a king move that doesn't capture.
    /// - A castling move is always contained to one rank.
    /// - A promotion is a pawn move.
    /// - A pawn-special move is a pawn move.
    /// - A pawn-special capture always captures a pawn.
    /// - A pawn-special non-capture is always 2 squares.
    /// - A pawn move non-capture is always on the same file.
    /// - A rook only moves orthogonally
    ///
    /// Todo:
    ///
    /// - Bishops always move diagonally
    /// -
    pub fn sanity_check(self) {
        if CastlingDirection::from_special(self.special).is_some() {
            assert_eq!(self.ech, ChessPiece::KING);
            assert_eq!(self.capture, None);
            assert_eq!(self.from as u8 & 0x7, self.to as u8 & 0x7);
        }

        if ChessPawn::from_special(self.special).is_some() {
            assert_eq!(self.ech, ChessPiece::PAWN);
            if self.capture.is_some() {
                assert_eq!(self.capture, Some(ChessCommoner::PAWN));
            } else {
                assert_eq!(self.from.ix().abs_diff(self.to.ix()), 16);
            }
        }

        if PawnPromotion::from_special(self.special).is_some() {
            assert_eq!(self.ech, ChessPiece::PAWN);
        }

        assert_ne!(self.from, self.to)
    }
}

/// The 'ply' identifier of a chess game.
///
/// In game theory, a ply is the general name for a single action
/// that a player performs when they take their 'turn'. In chess,
/// plies are uniquely identified by turn number and player color.
///
/// The turn number starts at 1 and increments after black has moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ply(
    /// Turn counter
    pub u16,
    /// Active player
    pub ChessColor,
);

impl Ply {
    /// Get the previous ply
    fn prev(self) -> Self {
        if self.1.is_white() {
            Self(self.0 - 1, ChessColor::BLACK)
        } else {
            Self(self.0, ChessColor::WHITE)
        }
    }

    /// Get the next ply
    fn next(self) -> Self {
        if self.1.is_black() {
            Self(self.0 + 1, ChessColor::WHITE)
        } else {
            Self(self.0, ChessColor::BLACK)
        }
    }
}

/// Representations of the transient metadata of a chessboard.
///
/// That is, information that is not readily apparent when observing
/// a chess position, and which is destroyed by certain moves. These
/// values can only be determined by examining the full move history.
///
/// In particular:
///
/// - Whether en-passant capture is possible, information which is lost
///   after the next move.
/// - Castling rights, which are lost upon any king move, or when a rook
///   is moved or captured (to that side only.)
/// - The number of half-moves that have happened since an irreversible
///   move, that is, capture or pawn push, for the purposes of the 50-move
///   draw rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transients {
    /// En-passant capture information.
    pub en_passant: Option<EnPassant>,
    /// Number of half-moves elapsed since last capture or pawn push.
    pub halfmove_clock: u8,
    /// Castling rights, indexed first by [`ChessColor`] then [`CastlingDirection`].
    pub rights: [[bool; 2]; 2],
}

impl Transients {
    /// Transients at the starting position of a standard chessboard
    pub fn startpos() -> Self {
        Self {
            en_passant: None,
            halfmove_clock: 0,
            rights: [[true; 2]; 2],
        }
    }

    /// Transients of an empty chessboard
    pub fn empty() -> Self {
        Self {
            en_passant: None,
            halfmove_clock: 0,
            rights: [[false; 2]; 2],
        }
    }
}

/// Representation of the en-passant capture rule.
///
/// En-passant capture is a special pawn capture, where
/// a pawn moving two squares as its initial move can be
/// captured by an enemy pawn on an immediately adjacent square
/// on the same rank.
///
/// This rule exists in tandem with the rule allowing pawns to
/// move two squares as their first move, to prevent the unopposed
/// creation of passed pawns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnPassant {
    /// Square upon which en-passant capture is possible.
    pub square: Square,
    /// Square of the captured pawn.
    pub capture: Square,
}

impl EnPassant {
    #[inline]
    pub fn bit_sq(this: Option<Self>) -> (u64, Option<Square>) {
        if let Some(this) = this {
            (1 << this.square.ix(), Some(this.square))
        } else {
            (0, None)
        }
    }
}

/// Representation of castling.
///
/// This struct is a data representation of the castling moves,
/// for the purposes of randomized chess variants such as Chess960
/// and Chess480.
///
/// The arrays are given as first indexed by color, then by direction.
#[derive(Debug)]
pub struct CastlingRules {
    /// Starting squares of the rooks
    pub rook_start: [[Square; 2]; 2],
    /// Ending squares of the rooks
    pub rook_end: [[Square; 2]; 2],
    /// Starting square of the king (there's only one)
    pub king_start: [Square; 2],
    /// Ending squares of the king
    pub king_end: [[Square; 2]; 2],
    /// Moves should generate with castling being a capture of
    /// one's own rook, rather than a 2-square move of the king
    pub capture_own_rook: bool,
}

impl CastlingRules {
    pub const STANDARD: CastlingRules = CastlingRules {
        rook_start: [[Square::a1, Square::h1], [Square::a8, Square::h8]],
        rook_end: [[Square::d1, Square::f1], [Square::d8, Square::f8]],
        king_start: [Square::e1, Square::e8],
        king_end: [[Square::c1, Square::g1], [Square::c8, Square::g8]],
        capture_own_rook: false,
    };

    pub fn chess_960(starting_array: [ChessOfficer; 8]) -> Self {
        todo!()
    }
}

/// Data for each square on the board
///
/// This is the basis of the simple and most obvious representation,
/// using a separate value in an array for each square, a so-called
/// 'board'-centric representation, which is `DataBoard<Option<ChessMan>>`
///
/// This is a generalized version allowing any values, not just
/// chessmen to fill the squares, which can be used for a variety
/// of purposes, such as conveniently setting up positions for more advanced
/// board representations.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct DataBoard<T>(pub [T; 64]);

impl<T> DataBoard<T> {
    /// Write to a square
    pub fn set(&mut self, sq: Square, it: T) {
        self.0[sq.ix()] = it
    }
}
