//! Making moves.
//!
//! See [`crate::model::BitMove`].
//!
//! Out move model accounts for four cases:
//! - the move is a simple move, see [`simple_move`]
//! - the move is a promotion move, see [`promotion_move`]
//! - the move is a special pawn move interacting with the _en passant_ rule, see [`pawn_special`]
//! - the move is a castling move, see [`castling_move`]
//!
//! Additionally:
//!
//! - a simple and promotion and special pawn move can be with captures, see [`capturing_move`]
//! - a move that either moves or captures a rook results in a loss of castling rights, see [`rook_rights_loss`]
//!
//! These are combined into all the utilities needed to make moves on a chessboard, and
//! are implemented using the [`BitBoard`] trait as a visitor pattern. Two additonal visitors
//! are supplied, for purely hashing, and for only moving without metadata updates.

use std::{hash::Hash, marker::PhantomData};

use strum::VariantArray;

use crate::model::{
    BitMove, CastlingDirection, ChessColor, ChessEchelon, ChessPawn, ChessPiece, EnPassant,
    LegalMove, PawnPromotion, PseudoLegal, SpecialMove, Square, Transients,
    board::{BitBoard, ChessBoard, MetaBoard},
    castling::{CLASSIC_CASTLING, Castling},
    hash::{NoHashes, ZobristTables},
    notation::{AlgNotaion, CoordNotation},
};

/// Make a legal move on a bitboard given a Zobrist hashing table
///
/// If `mv` is not a legal move that was just generated in accordance with `board`,
/// the behavior is unspecified.
pub fn make_legal_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: LegalMove,
) -> Transients {
    let zobristhashes = ZT::static_table();

    let res = board.trans();

    simple_move(board, mv.0, zobristhashes);
    promotion_move(board, mv.0, zobristhashes);
    pawn_special(board, mv.0, zobristhashes);
    castling_move(board, mv.0, zobristhashes);

    board.next_ply();

    return res;
}

/// Unmake a legal move just made.
///
/// If `mv` is not a move just made on `board` using [`make_legal_move`] and `trans` is not
/// the result of calling [`make_legal_move`], then the behavior is unspecified.
pub fn unmake_legal_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: LegalMove,
    trans: Transients,
) {
    let zobristhashes = ZT::static_table();

    board.set_transients(trans);

    board.prev_ply();

    simple_move(board, mv.0, zobristhashes);
    promotion_move(board, mv.0, zobristhashes);
    pawn_special(board, mv.0, zobristhashes);
    castling_move(board, mv.0, zobristhashes);

    board.set_transients(trans);
}

/// Clone the board and make the legal move on the clone.
pub fn clone_make_legal_move<BB: BitBoard + Clone, ZT: ZobristTables>(
    board: &BB,
    mv: LegalMove,
) -> BB {
    let mut res = board.clone();
    make_legal_move::<BB, ZT>(&mut res, mv);
    res
}

/// Clone the board and make the pseudo-legal move on the clone, without
/// accounting for anything except the board position.
///
/// This can be used for analyzing whether a move puts the king in check.
pub fn clone_make_pseudolegal_move<BB: BitBoard + Clone>(board: &BB, mv: PseudoLegal) -> BB {
    let mut res = board.clone();
    make_legal_move::<MoveOnly<BB>, NoHashes>(&mut MoveOnly(&mut res), LegalMove(mv.0));
    res
}

/// Compute the zobrist hash that would result from applying this move.
pub fn hash_prospective_move<BB: BitBoard, ZT: ZobristTables>(board: &BB, mv: BitMove) -> u64 {
    let mut res = HashOnly(
        board.curr_hash(),
        board.trans(),
        board.ply().0,
        board.castling(),
    );
    make_legal_move::<HashOnly, ZT>(&mut res, LegalMove(mv));
    res.0
}

/// Make a simple move on a chessboard:
///
/// - A chessman moves from one square to another
/// - Optionally captures a chessman of the opposing color at its destination
/// - A forefit of castling rights may occur if a rook moves from its starting square
#[inline]
pub fn simple_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: BitMove,
    zobristhashes: &'static ZT,
) {
    let player = board.ply().0;

    board.set_halfmove_clock(board.trans().halfmove_clock + 1);

    if mv.special.is_some() {
        return;
    }

    let bits = (1 << mv.from as u8) | (1 << mv.to as u8);

    board.xor(player, mv.ech, bits);

    rook_rights_loss(board, mv.ech, player, mv.from, zobristhashes);
    capturing_move(board, mv, mv.to, zobristhashes);

    board.hash(zobristhashes.hash_move(player, mv.ech, bits));
}

/// The loss of rook rights, either from having a rook captured, or moving it from its starting square.
#[inline]
pub fn rook_rights_loss<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    piece: ChessEchelon,
    color: ChessColor,
    sq: Square,
    zobristhashes: &'static ZT,
) {
    use CastlingDirection::*;
    if piece != ChessEchelon::ROOK {
        return;
    }

    let castling = board.castling();

    for dir in [EAST, WEST] {
        if sq == castling.rook_start[color.ix()][dir.ix()] {
            let mut rights = board.trans().rights;

            board.hash(zobristhashes.hash_rights(rights));

            rights[color.ix()][dir.ix()] = false;

            board.set_castling_rights(rights);
            board.hash(zobristhashes.hash_rights(rights));
        }
    }
}

/// A capturing move:
///
/// - A chessman of opposing color disappears from the board
/// - A loss of castling rights occur if the captured piece is a rook on its starting square
#[inline]
pub fn capturing_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: BitMove,
    sq: Square,
    zobristhashes: &'static ZT,
) {
    let opponent = board.ply().0.opp();

    let Some(man) = mv.capture else {
        return;
    };

    let man = ChessEchelon::from(man);

    board.set_halfmove_clock(0);
    board.xor(opponent, man, 1 << sq.ix());

    rook_rights_loss(board, man, opponent, mv.to, zobristhashes);

    board.hash(zobristhashes.hash_square(opponent, man, sq));
}

/// Make a pawn special move:
///
/// - _En passant_ capture, or
/// - Double-push incurring an _en passant_ vulnerability
///
/// Also clears the _en passant_ information for all moves
#[inline]
pub fn pawn_special<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: BitMove,
    zobristhashes: &'static ZT,
) {
    let player = board.ply().0;

    let en_passant = board.trans().en_passant;

    board.set_en_passant(None);
    if mv.ech == ChessEchelon::PAWN {
        board.set_halfmove_clock(0);
    }

    board.hash(zobristhashes.hash_en_passant(en_passant));

    let Some(_) = ChessPawn::from_special(mv.special) else {
        return;
    };

    let bits = 1 << mv.from.ix() | 1 << mv.to.ix();

    board.xor(player, ChessEchelon::PAWN, bits);

    board.hash(zobristhashes.hash_move(player, ChessEchelon::PAWN, bits));

    if let Some(en_passant) = en_passant {
        capturing_move(board, mv, en_passant.capture, zobristhashes);
    }

    if (mv.from as u8).abs_diff(mv.to as u8) == 16 {
        let en_passant = Some(EnPassant {
            capture: mv.to,
            square: Square::from_u8((mv.from as u8).min(mv.to as u8) + 8),
        });

        board.set_en_passant(en_passant);

        board.hash(zobristhashes.hash_en_passant(en_passant));
    }
}

/// A pawn promotion move:
///
/// - A pawn moves or captures onto the enemy back rank
/// - The pawn disappears
/// - A new non-king piece appears on its destination square
#[inline]
pub fn promotion_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: BitMove,
    zobristhashes: &'static ZT,
) {
    let player = board.ply().0;

    let Some(prom) = PawnPromotion::from_special(mv.special) else {
        return;
    };
    let prom = ChessEchelon::from(prom);

    board.xor(player, ChessEchelon::PAWN, 1 << mv.from.ix());
    board.xor(player, prom, 1 << mv.to.ix());

    capturing_move(board, mv, mv.to, zobristhashes);

    board.hash(zobristhashes.hash_square(player, ChessEchelon::PAWN, mv.from));
    board.hash(zobristhashes.hash_square(player, prom, mv.to));
}

/// A castling move:
///
/// - The king and rook both move
/// - No captures occur
/// - Expenditure of castling rights
///
/// This function also forefeits castling rights if the king moves in general.
#[inline]
pub fn castling_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: BitMove,
    zobristhashes: &'static ZT,
) {
    let player = board.ply().0;

    let Some(castle) = CastlingDirection::from_special(mv.special) else {
        return;
    };

    let back_rank = board.castling().back_rank[player.ix()];
    let king_move = board.castling().king_move[castle.ix()] & back_rank;
    let rook_move = board.castling().rook_move[castle.ix()] & back_rank;

    let mut rights = board.trans().rights;

    board.hash(zobristhashes.hash_rights(rights));

    rights[player.ix()] = [false; 2];

    board.hash(zobristhashes.hash_rights(rights));

    board.set_halfmove_clock(board.trans().halfmove_clock + 1);
    board.xor(player, ChessEchelon::KING, king_move);
    board.xor(player, ChessEchelon::ROOK, rook_move);
    board.set_castling_rights(rights);

    board.hash(zobristhashes.hash_castling(player, king_move, rook_move));
}

/// A wrapper for a [`BitBoard`]-type which only makes moves, without updating metadata or hashing.
#[repr(transparent)]
pub struct MoveOnly<'a, BB: BitBoard>(pub &'a mut BB);

impl<'a, BB: BitBoard> Clone for MoveOnly<'a, BB> {
    fn clone(&self) -> Self {
        panic!("Not actually cloneable")
    }
}

impl<'a, BB: BitBoard> MetaBoard for MoveOnly<'a, BB> {
    #[inline]
    fn trans(&self) -> Transients {
        self.0.trans()
    }

    #[inline]
    fn set_halfmove_clock(&mut self, val: u8) {
        // self.0.set_halfmove_clock(val)
    }

    #[inline]
    fn set_castling_rights(&mut self, rights: [[bool; 2]; 2]) {
        // self.0.set_castling_rights(rights);
    }

    #[inline]
    fn set_en_passant(&mut self, eps: Option<EnPassant>) {
        // self.0.set_en_passant(eps);
    }

    #[inline]
    fn set_transients(&mut self, trans: Transients) {
        // self.0.set_transients(trans)
    }

    #[inline]
    fn curr_hash(&self) -> u64 {
        0
    }

    #[inline]
    fn hash(&mut self, hash: u64) {}

    #[inline]
    fn ply(&self) -> (ChessColor, u16) {
        self.0.ply()
    }

    #[inline]
    fn next_ply(&mut self) {}

    #[inline]
    fn prev_ply(&mut self) {}

    #[inline]
    fn castling(&self) -> &'static super::castling::Castling {
        self.0.castling()
    }
}

impl<'a, BB: BitBoard> ChessBoard for MoveOnly<'a, BB> {
    fn startpos<ZT: ZobristTables>() -> Self {
        panic!("Not implemented.");
    }

    #[inline]
    fn sanity_check<ZT: ZobristTables>(&self) {}

    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        0
    }

    fn empty() -> Self {
        panic!("Not implemented.");
    }
}

impl<'a, BB: BitBoard> BitBoard for MoveOnly<'a, BB> {
    #[inline]
    fn xor(&mut self, color: ChessColor, ech: ChessEchelon, mask: u64) {
        self.0.xor(color, ech, mask);
    }

    #[inline]
    fn men(&self, color: ChessColor, ech: ChessEchelon) -> u64 {
        0
    }

    #[inline]
    fn color(&self, color: ChessColor) -> u64 {
        0
    }

    #[inline]
    fn total(&self) -> u64 {
        0
    }

    #[inline]
    fn ech_at(&self, sq: Square) -> Option<ChessEchelon> {
        None
    }

    #[inline]
    fn side(&self, color: ChessColor) -> std::borrow::Cow<'_, [u64; 6]> {
        std::borrow::Cow::Owned([0; 6])
    }

    #[inline]
    fn comm_at(&self, sq: Square) -> Option<super::ChessCommoner> {
        None
    }
}

/// An empty [`BitBoard`]-type which only hashes, without updating metadata or moving.
#[derive(Debug, Clone, Copy)]
pub struct HashOnly(
    pub u64,
    pub Transients,
    pub ChessColor,
    pub &'static Castling,
);

impl MetaBoard for HashOnly {
    #[inline]
    fn trans(&self) -> Transients {
        self.1
    }

    #[inline]
    fn curr_hash(&self) -> u64 {
        self.0
    }

    #[inline]
    fn hash(&mut self, hash: u64) {
        self.0 ^= hash
    }

    #[inline]
    fn ply(&self) -> (ChessColor, u16) {
        (self.2, 0)
    }

    #[inline]
    fn next_ply(&mut self) {}

    #[inline]
    fn prev_ply(&mut self) {}

    #[inline]
    fn castling(&self) -> &'static super::castling::Castling {
        self.3
    }

    #[inline]
    fn set_halfmove_clock(&mut self, val: u8) {}

    #[inline]
    fn set_castling_rights(&mut self, rights: [[bool; 2]; 2]) {}

    #[inline]
    fn set_en_passant(&mut self, rights: Option<EnPassant>) {}

    #[inline]
    fn set_transients(&mut self, trans: Transients) {}
}

impl ChessBoard for HashOnly {
    #[inline]
    fn startpos<ZT: ZobristTables>() -> Self {
        Self(
            0,
            Transients::startpos(),
            ChessColor::WHITE,
            &CLASSIC_CASTLING,
        )
    }

    #[inline]
    fn sanity_check<ZT: ZobristTables>(&self) {}

    #[inline]
    fn rehash<ZT: ZobristTables>(&self) -> u64 {
        self.0
    }

    #[inline]
    fn empty() -> Self {
        Self(0, Transients::empty(), ChessColor::WHITE, &CLASSIC_CASTLING)
    }
}

impl BitBoard for HashOnly {
    #[inline]
    fn xor(&mut self, color: ChessColor, ech: ChessEchelon, mask: u64) {}

    #[inline]
    fn men(&self, color: ChessColor, ech: ChessEchelon) -> u64 {
        0
    }

    #[inline]
    fn color(&self, color: ChessColor) -> u64 {
        0
    }

    #[inline]
    fn total(&self) -> u64 {
        0
    }

    #[inline]
    fn ech_at(&self, sq: Square) -> Option<ChessEchelon> {
        None
    }

    #[inline]
    fn side(&self, color: ChessColor) -> std::borrow::Cow<'_, [u64; 6]> {
        std::borrow::Cow::Owned([0; 6])
    }

    #[inline]
    fn comm_at(&self, sq: Square) -> Option<super::ChessCommoner> {
        self.ech_at(sq).and_then(super::ChessCommoner::from_echelon)
    }
}
