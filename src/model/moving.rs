use std::{hash::Hash, marker::PhantomData};

use strum::VariantArray;

use crate::model::{
    BitMove, CastlingDirection, ChessColor, ChessEchelon, ChessPawn, ChessPiece, ChessPromotion,
    EnPassant, Legal, PseudoLegal, SpecialMove, Square, Transients,
    bitboard::{BitBoard, ChessBoard, MetaBoard},
    castling::Castling,
    hash::{NoHashes, ZobristTables},
    notation::{AlgNotaion, CoordNotation},
};

pub fn make_legal_move<BB: BitBoard, ZT: ZobristTables>(board: &mut BB, mv: Legal) -> Transients {
    let zobristhashes = ZT::static_table();

    let res = board.trans();

    simple_move(board, mv.0, zobristhashes);
    promotion_move(board, mv.0, zobristhashes);
    pawn_special(board, mv.0, zobristhashes);
    castling_move(board, mv.0, zobristhashes);

    board.next_ply();

    return res;
}

pub fn unmake_legal_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: Legal,
    trans: Transients,
) {
    let zobristhashes = ZT::static_table();

    board.prev_ply();

    simple_move(board, mv.0, zobristhashes);
    promotion_move(board, mv.0, zobristhashes);
    pawn_special(board, mv.0, zobristhashes);
    castling_move(board, mv.0, zobristhashes);

    board.set_castling_rights(trans.rights);
    board.set_halfmove_clock(trans.halfmove_clock);
    board.set_en_passant(trans.en_passant);
}

pub fn fake_move<BB: BitBoard>(board: &mut BB, mv: PseudoLegal) {
    make_legal_move::<MoveOnly<BB>, NoHashes>(&mut MoveOnly(board), Legal(mv.0));
}

pub fn hash_move<BB: BitBoard, ZT: ZobristTables>(board: &BB, mv: PseudoLegal) {
    make_legal_move::<HashOnly, ZT>(
        &mut HashOnly(0, board.trans(), board.ply().0, board.castling()),
        Legal(mv.0),
    );
}

fn simple_move<BB: BitBoard, ZT: ZobristTables>(
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
    capture(board, mv, mv.to, zobristhashes);

    board.hash(zobristhashes.hash_move(player, mv.ech, bits));
}

#[inline]
fn rook_rights_loss<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    piece: ChessEchelon,
    color: ChessColor,
    sq: Square,
    zobristhashes: &'static ZT,
) {
    if piece != ChessEchelon::ROOK {
        return;
    }

    for dir in [CastlingDirection::EAST, CastlingDirection::WEST] {
        if sq == board.castling().rook_from[dir.ix()] {
            let mut rights = board.trans().rights;

            board.hash(zobristhashes.hash_rights(rights));

            rights[color.ix()][dir.ix()] = false;

            board.set_castling_rights(rights);
            board.hash(zobristhashes.hash_rights(rights));
        }
    }
}

#[inline]
fn capture<BB: BitBoard, ZT: ZobristTables>(
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

#[inline]
fn pawn_special<BB: BitBoard, ZT: ZobristTables>(
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
        capture(board, mv, en_passant.capture, zobristhashes);
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

#[inline]
fn promotion_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: BitMove,
    zobristhashes: &'static ZT,
) {
    let player = board.ply().0;

    let Some(prom) = ChessPromotion::from_special(mv.special) else {
        return;
    };
    let prom = ChessEchelon::from(prom);

    board.xor(player, ChessEchelon::PAWN, 1 << mv.from.ix());
    board.xor(player, prom, 1 << mv.to.ix());

    capture(board, mv, mv.to, zobristhashes);

    board.hash(zobristhashes.hash_square(player, ChessEchelon::PAWN, mv.from));
    board.hash(zobristhashes.hash_square(player, prom, mv.to));
}

#[inline]
fn castling_move<BB: BitBoard, ZT: ZobristTables>(
    board: &mut BB,
    mv: BitMove,
    zobristhashes: &'static ZT,
) {
    let player = board.ply().0;

    let Some(castle) = CastlingDirection::from_special(mv.special) else {
        return;
    };

    let rank = if player.is_black() {
        0xFF00_0000_0000_0000
    } else {
        0x0000_0000_0000_00FF
    };

    let king_move = board.castling().king_move[castle.ix()] & rank;
    let rook_move = board.castling().rook_move[castle.ix()] & rank;

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

struct MoveOnly<'a, BB: BitBoard>(&'a mut BB);

impl<'a, BB: BitBoard> MetaBoard for MoveOnly<'a, BB> {
    #[inline]
    fn trans(&self) -> Transients {
        self.0.trans()
    }

    #[inline]
    fn set_halfmove_clock(&mut self, val: u8) {}

    #[inline]
    fn set_castling_rights(&mut self, rights: [[bool; 2]; 2]) {
        // self.0.set_castling_rights(rights);
    }

    #[inline]
    fn set_en_passant(&mut self, eps: Option<EnPassant>) {
        // self.0.set_en_passant(eps);
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
}

struct HashOnly(u64, Transients, ChessColor, &'static Castling);

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
}

impl BitBoard for HashOnly {
    fn xor(&mut self, color: ChessColor, ech: ChessEchelon, mask: u64) {}

    fn men(&self, color: ChessColor, ech: ChessEchelon) -> u64 {
        0
    }

    fn color(&self, color: ChessColor) -> u64 {
        0
    }

    fn total(&self) -> u64 {
        0
    }
}
