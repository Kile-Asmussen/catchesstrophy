use std::ops::{FromResidual, Try};

use strum::VariantArray;

use crate::model::{
    BitBoard, BitMove, Castles, ChessMan, ChessPawn, Color, EnPassant, Legal, Promotion, Rights,
    Special, Square, TransientInfo, ZOBRISTHASHES, ZobristHashes,
    notation::{AlgNotaion, CoordNotation},
};

impl BitBoard {
    pub fn rehash(&self) -> u64 {
        use ChessMan::*;
        use Color::*;

        let mut res = 0;

        let zobristhashes = &*ZOBRISTHASHES;

        for piece in [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING] {
            res ^= zobristhashes.hash_piece_mask(piece, self.pieces[piece as usize - 1]);
        }

        for color in [WHITE, BLACK] {
            res ^= zobristhashes.hash_color_mask(color, self.colors[color as usize]);
        }

        res ^= zobristhashes.hash_rights(self.trans.rights);

        if let Some(ep) = self.trans.en_passant {
            res ^= zobristhashes.hash_file(ep.square as u8);
        }

        if self.player == BLACK {
            res ^= zobristhashes.black_to_move;
        }

        res
    }

    pub fn make_move(&mut self, mv: Legal) -> TransientInfo {
        let res = self.trans;
        let zobristhashes = &*&ZOBRISTHASHES;

        self.update_transient::<true>(mv.0, zobristhashes);

        self.simple_move::<true>(mv.0, zobristhashes);
        self.promotion_move::<true>(mv.0, zobristhashes);
        self.pawn_special_move::<true>(mv.0, zobristhashes);
        self.castling_move::<true>(mv.0, zobristhashes);

        res
    }

    pub(crate) fn fake_move(&mut self, mv: BitMove) {
        todo!()
    }

    pub fn unmake_move(&mut self, mv: Legal, trans: TransientInfo) {
        todo!()
    }

    pub fn null_move(&mut self) {
        self.player = self.player.opp();
        self.hash ^= ZOBRISTHASHES.black_to_move;
    }

    #[inline]
    fn simple_move<const HASH: bool>(&mut self, mv: BitMove, zobristhashes: &ZobristHashes) {
        if mv.special.is_some() {
            return;
        }

        let bits = (1 << mv.from as u8) | (1 << mv.to as u8);

        self.pieces[mv.piece.ix()] ^= bits;
        self.colors[self.player.ix()] ^= bits;

        if HASH {
            self.hash ^= zobristhashes.hash_piece_mask(mv.piece, bits);
            self.hash ^= zobristhashes.hash_color_mask(self.player, bits);
        }

        self.capture::<{ HASH }>(mv, mv.to, zobristhashes);
    }

    #[inline]
    fn promotion_move<const HASH: bool>(&mut self, mv: BitMove, zobristhashes: &ZobristHashes) {
        let Some(prom) = Promotion::from_special(mv.special) else {
            return;
        };

        debug_assert_eq!(mv.piece, ChessMan::PAWN);

        self.pieces[ChessPawn::PAWN.ix()] ^= 1 << mv.from as u8;
        self.pieces[prom.ix()] ^= 1 << mv.to as u8;

        if HASH {
            self.hash ^= zobristhashes.pieces[ChessPawn::PAWN.ix()][mv.from.ix()];
            self.hash ^= zobristhashes.pieces[prom.ix()][mv.to.ix()];
        }

        self.capture::<{ HASH }>(mv, mv.to, zobristhashes);
    }

    #[inline]
    fn castling_move<const HASH: bool>(&mut self, mv: BitMove, zobristhashes: &ZobristHashes) {
        let Some(castle) = Castles::from_special(mv.special) else {
            return;
        };

        debug_assert_eq!(mv.piece, ChessMan::KING);

        let rank = if self.player.is_black() {
            0xFF00_0000_0000_0000
        } else {
            0x0000_0000_0000_00FF
        };
        let king_move = self.castling.king_move[castle.ix()] & rank;
        let rook_move = self.castling.rook_move[castle.ix()] & rank;

        self.pieces[ChessMan::KING as usize - 1] ^= king_move;
        self.pieces[ChessMan::ROOK as usize - 1] ^= rook_move;
        self.colors[self.player as usize] ^= king_move | rook_move;

        if HASH {
            self.hash ^= zobristhashes.hash_piece_mask(ChessMan::KING, king_move);
            self.hash ^= zobristhashes.hash_piece_mask(ChessMan::ROOK, rook_move);
            self.hash ^= zobristhashes.hash_color_mask(self.player, king_move | rook_move);
        }
    }

    #[inline]
    fn pawn_special_move<const HASH: bool>(&mut self, mv: BitMove, zobristhashes: &ZobristHashes) {
        if mv.special != Some(Special::PAWN) {
            self.trans.en_passant = None;
            return;
        }

        debug_assert_eq!(mv.piece, ChessMan::PAWN);

        if let Some(ep) = self.trans.en_passant {
            self.capture::<{ HASH }>(mv, ep.capture, zobristhashes);
            self.trans.en_passant = None;
        }

        if (mv.from as u8).abs_diff(mv.to as u8) != 16 {
            self.trans.en_passant = None;
            return;
        }

        let ix = (mv.from as u8).min(mv.to as u8) + 8;
        self.trans.en_passant = Some(EnPassant {
            capture: mv.to,
            square: Square::from_u8(ix),
        });

        if HASH {
            self.hash ^= zobristhashes.hash_file(ix);
        }
    }

    #[inline]
    fn capture<const HASH: bool>(
        &mut self,
        mv: BitMove,
        sq: Square,
        zobristhashes: &ZobristHashes,
    ) {
        let Some(man) = mv.capture else {
            return;
        };

        self.pieces[man.ix()] ^= 1 << sq.ix();
        self.colors[self.player.opp().ix()] ^= 1 << sq.ix();

        if HASH {
            self.hash ^= zobristhashes.pieces[man.ix()][sq.ix()];
            self.hash ^= zobristhashes.colors[self.player.opp().ix()][sq.ix()];
        }
    }

    #[inline]
    fn update_transient<const HASH: bool>(&mut self, mv: BitMove, zobristhashes: &ZobristHashes) {
        let ix = self.player.ix() << 1;

        if HASH {
            self.hash ^= zobristhashes.hash_rights(self.trans.rights);
        }

        if mv.piece == ChessMan::KING {
            let bits = 0x3 << ix;
            self.trans.rights.0 &= !bits;
        }

        for (piece, square, color) in [
            (Some(mv.piece), mv.from, self.player),
            (mv.capture.map(ChessMan::from), mv.to, self.player.opp()),
        ] {
            if piece == Some(ChessMan::ROOK) {
                for dir in [Castles::EAST, Castles::WEST] {
                    if square == self.castling.rook_from[dir.ix()] {
                        let ix = color.ix() << 1 | dir.ix();
                        self.trans.rights.0 &= !(1 << ix);
                    }
                }
            }
        }

        if HASH {
            self.hash ^= zobristhashes.hash_rights(self.trans.rights);
        }

        if mv.capture.is_some() || mv.piece == ChessMan::PAWN {
            self.trans.halfmove_clock = 0;
        } else {
            self.trans.halfmove_clock += 1;
        }

        if HASH {
            if let Some(ep_square) = self.trans.en_passant {
                self.hash ^= zobristhashes.hash_file(ep_square.square as u8);
            }
        }
    }
}

impl ZobristHashes {
    #[inline]
    pub fn hash_mask(&self, board: &[u64; 64], mut mask: u64) -> u64 {
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            let bit = 1 << sq;
            mask ^= bit;
            res ^= board[sq as usize & 0x3F];
        }
        res
    }

    #[inline]
    pub fn hash_color_mask(&self, color: Color, mut mask: u64) -> u64 {
        let board = &self.colors[color as usize];
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            let bit = 1 << sq;
            mask ^= bit;
            res ^= board[sq as usize & 0x3F];
        }
        res
    }

    #[inline]
    pub fn hash_piece_mask(&self, piece: ChessMan, mut mask: u64) -> u64 {
        let board = &self.pieces[piece as usize - 1];
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            let bit = 1 << sq;
            mask ^= bit;
            res ^= board[sq as usize & 0x3F];
        }
        res
    }

    #[inline]
    pub fn hash_rights(&self, r: Rights) -> u64 {
        let mut res = 0;
        for ix in 0..=3 {
            if r.0 & 1 << ix != 0 {
                res ^= self.castling[ix];
            }
        }
        res
    }

    #[inline]
    pub fn hash_file(&self, ix: u8) -> u64 {
        self.ep_file[ix as usize & 0x7]
    }
}

trait MoveMatcher {
    fn matches(self, mv: BitMove) -> bool;
}

impl MoveMatcher for CoordNotation {
    fn matches(self, mv: BitMove) -> bool {
        self.from == mv.from
            && self.to == mv.to
            && (self.prom.is_none() || self.prom == Promotion::from_special(mv.special))
    }
}

impl MoveMatcher for AlgNotaion {
    fn matches(self, mv: BitMove) -> bool {
        match self {
            Self::Pawn(p, _) => {
                mv.piece == ChessMan::PAWN
                    && (p.from as u8 & 0x7) == (mv.from as u8 & 0x7)
                    && p.to == mv.to
                    && p.capture == mv.capture.is_some()
                    && p.promote == Promotion::from_special(mv.special)
            }
            Self::Piece(p, _) => {
                mv.piece == ChessMan::from(p.piece)
                    && mv.to == p.to
                    && p.capture == mv.capture.is_some()
                    && match p.disambiguate {
                        (false, false) => true,
                        (true, false) => (p.from as u8 & 0x7) == (mv.from as u8 & 0x7),
                        (false, true) => (p.from as u8 & 0x38) == (mv.from as u8 & 0x38),
                        (true, true) => p.from == mv.from,
                    }
            }
            Self::Caslte(c, _) => mv.special == Some(Special::from(c)),
        }
    }
}
