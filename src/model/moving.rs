use std::{
    marker::PhantomData,
    ops::{FromResidual, Try},
};

use strum::VariantArray;

use crate::model::{
    BitMove, Castles, ChessMan, ChessPawn, ChessPiece, Color, EnPassant, Legal, Promotion, Special,
    Square, Transients,
    bitboard::BitBoard,
    hash::{NoHashes, ZobristTables},
    notation::{AlgNotaion, CoordNotation},
};

#[allow(private_bounds)]
pub trait BitBoardMoves: BitBoardMoveComponents {
    fn make_move<ZT: ZobristTables>(&mut self, mv: Legal) -> Transients;
    fn unmake_move<ZT: ZobristTables>(&mut self, mv: Legal, trans: Transients);
    fn hash_move<ZT: ZobristTables>(&mut self, mv: BitMove) -> u64;
    fn fake_move(&mut self, mv: BitMove);
}

impl<BB: BitBoardMoveComponents> BitBoardMoves for BB {
    fn make_move<ZT: ZobristTables>(&mut self, mv: Legal) -> Transients {
        let zobristhashes = ZT::static_table();

        let res = self.trans();

        self.simple_move::<true, true, ZT>(mv.0, zobristhashes);
        self.promotion_move::<true, true, ZT>(mv.0, zobristhashes);
        self.pawn_special::<true, true, ZT>(mv.0, zobristhashes);
        self.castling_move::<true, true, ZT>(mv.0, zobristhashes);

        self.next_ply();

        return res;
    }

    fn unmake_move<ZT: ZobristTables>(&mut self, mv: Legal, trans: Transients) {
        let zobristhashes = ZT::static_table();

        self.prev_ply();

        self.simple_move::<true, true, ZT>(mv.0, zobristhashes);
        self.promotion_move::<true, true, ZT>(mv.0, zobristhashes);
        self.pawn_special::<true, true, ZT>(mv.0, zobristhashes);
        self.castling_move::<true, true, ZT>(mv.0, zobristhashes);

        *self.trans_mut() = trans;
    }

    fn hash_move<ZT: ZobristTables>(&mut self, mv: BitMove) -> u64 {
        let zobristhashes = ZT::static_table();

        self.simple_move::<false, true, ZT>(mv, zobristhashes)
            ^ self.promotion_move::<false, true, ZT>(mv, zobristhashes)
            ^ self.pawn_special::<false, true, ZT>(mv, zobristhashes)
            ^ self.castling_move::<false, true, ZT>(mv, zobristhashes)
    }

    fn fake_move(&mut self, mv: BitMove) {
        self.simple_move::<true, false, NoHashes>(mv, NoHashes::static_table());
        self.promotion_move::<true, false, NoHashes>(mv, NoHashes::static_table());
        self.pawn_special::<true, false, NoHashes>(mv, NoHashes::static_table());
        self.castling_move::<true, false, NoHashes>(mv, NoHashes::static_table());
    }
}

trait BitBoardMoveComponents: BitBoard {
    fn simple_move<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64;

    fn promotion_move<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64;

    fn castling_move<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64;

    fn capture<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        sq: Square,
        zobristhashes: &'static ZT,
    ) -> u64;

    fn rook_special<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        piece: ChessMan,
        color: Color,
        sq: Square,
        zobristhashes: &'static ZT,
    ) -> u64;

    fn pawn_special<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64;
}

impl<BB: BitBoard> BitBoardMoveComponents for BB {
    #[inline]
    fn simple_move<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64 {
        let mut hash = 0;
        let player = self.ply().0;

        if MUT {
            self.trans_mut().halfmove_clock += 1;
        }

        if mv.special.is_some() {
            return hash;
        }

        let bits = (1 << mv.from as u8) | (1 << mv.to as u8);

        if MUT {
            self.xor(player, mv.man, bits);
        }

        let rook_hash =
            self.rook_special::<{ MUT }, { HASH }, ZT>(mv.man, player, mv.from, zobristhashes);
        let cap_hash = self.capture::<{ MUT }, { HASH }, ZT>(mv, mv.to, zobristhashes);

        if HASH {
            hash ^= cap_hash;
            hash ^= rook_hash;
            hash ^= zobristhashes.hash_move(player, mv.man, bits);
        }

        if HASH && MUT {
            self.hash(hash);
        }

        return hash;
    }

    #[inline]
    fn promotion_move<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64 {
        let mut hash = 0;
        let player = self.ply().0;

        let Some(prom) = Promotion::from_special(mv.special) else {
            return hash;
        };
        let prom = ChessMan::from(prom);

        if MUT {
            self.xor(player, ChessMan::PAWN, 1 << mv.from.ix());
            self.xor(player, prom, 1 << mv.to.ix());
        }

        let cap_hash = self.capture::<{ MUT }, { HASH }, ZT>(mv, mv.to, zobristhashes);

        if HASH {
            hash ^= cap_hash;
            hash ^= zobristhashes.hash_square(player, ChessMan::PAWN, mv.from);
            hash ^= zobristhashes.hash_square(player, prom, mv.to);
        }

        if HASH && MUT {
            self.hash(hash);
        }

        return hash;
    }

    #[inline]
    fn castling_move<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64 {
        let mut hash = 0;
        let player = self.ply().0;

        let Some(castle) = Castles::from_special(mv.special) else {
            return hash;
        };

        let rank = if player.is_black() {
            0xFF00_0000_0000_0000
        } else {
            0x0000_0000_0000_00FF
        };

        let king_move = self.castling().king_move[castle.ix()] & rank;
        let rook_move = self.castling().rook_move[castle.ix()] & rank;

        let mut rights = self.trans().rights;

        if HASH {
            hash ^= zobristhashes.hash_rights(rights);
        }

        rights[player.ix()] = [false; 2];

        if MUT {
            self.trans().halfmove_clock += 1;
            self.xor(player, ChessMan::KING, king_move);
            self.xor(player, ChessMan::ROOK, rook_move);
            self.trans().rights = rights;
        }

        if HASH {
            hash ^= zobristhashes.hash_rights(rights);
            hash ^= zobristhashes.hash_castling(player, king_move, rook_move);
        }

        if HASH && MUT {
            self.hash(hash);
        }

        return hash;
    }

    #[inline]
    fn pawn_special<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        zobristhashes: &'static ZT,
    ) -> u64 {
        let mut hash = 0;
        let player = self.ply().0;

        let en_passant = self.trans().en_passant;

        if MUT {
            self.trans_mut().en_passant = None;
            if mv.man == ChessMan::PAWN {
                self.trans_mut().halfmove_clock = 0;
            }
        }

        if HASH {
            hash ^= zobristhashes.hash_en_passant(en_passant);
        }

        let Some(_) = ChessPawn::from_special(mv.special) else {
            return hash;
        };

        let bits = 1 << mv.from.ix() | 1 << mv.to.ix();

        if MUT {
            self.trans_mut().halfmove_clock = 0;
            self.xor(player, ChessMan::PAWN, bits);
        }

        if HASH {
            hash ^= zobristhashes.hash_move(player, ChessMan::PAWN, bits);
        }

        if let Some(en_passant) = en_passant {
            let cap_hash =
                self.capture::<{ MUT }, { HASH }, ZT>(mv, en_passant.capture, zobristhashes);

            if HASH {
                hash ^= cap_hash;
            }
        }

        if (mv.from as u8).abs_diff(mv.to as u8) == 16 {
            let en_passant = Some(EnPassant {
                capture: mv.to,
                square: Square::from_u8((mv.from as u8).min(mv.to as u8) + 8),
            });

            if MUT {
                self.trans_mut().en_passant = Some(EnPassant {
                    capture: mv.to,
                    square: Square::from_u8((mv.from as u8).min(mv.to as u8) + 8),
                });
            }

            if HASH {
                hash ^= zobristhashes.hash_en_passant(en_passant);
            }
        }

        if HASH && MUT {
            self.hash(hash);
        }

        return hash;
    }

    #[inline]
    fn capture<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        mv: BitMove,
        sq: Square,
        zobristhashes: &'static ZT,
    ) -> u64 {
        let mut hash = 0;
        let opponent = self.ply().0.opp();

        let Some(man) = mv.capture else {
            return hash;
        };
        let man = ChessMan::from(man);

        if MUT {
            self.trans_mut().halfmove_clock = 0;
            self.xor(opponent, man, 1 << sq.ix());
        }

        let rook_hash =
            self.rook_special::<{ MUT }, { HASH }, ZT>(man, opponent, mv.to, zobristhashes);

        if HASH {
            hash ^= rook_hash;
            hash ^= zobristhashes.hash_square(opponent, man, sq);
        }

        if HASH && MUT {
            self.hash(hash);
        }

        hash
    }

    fn rook_special<const MUT: bool, const HASH: bool, ZT: ZobristTables>(
        &mut self,
        piece: ChessMan,
        color: Color,
        sq: Square,
        zobristhashes: &'static ZT,
    ) -> u64 {
        let mut hash = 0;

        if piece != ChessMan::ROOK {
            return hash;
        }

        for dir in [Castles::EAST, Castles::WEST] {
            if sq == self.castling().rook_from[dir.ix()] {
                let mut rights = self.trans().rights;

                if HASH {
                    hash ^= zobristhashes.hash_rights(rights);
                }

                rights[color.ix()][dir.ix()] = false;

                if MUT {
                    self.trans_mut().rights = rights;
                }

                if HASH {
                    hash ^= zobristhashes.hash_rights(rights);
                }
            }
        }

        if HASH && MUT {
            self.hash(hash);
        }

        return hash;
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
                mv.man == ChessMan::PAWN
                    && (p.from as u8 & 0x7) == (mv.from as u8 & 0x7)
                    && p.to == mv.to
                    && p.capture == mv.capture.is_some()
                    && p.promote == Promotion::from_special(mv.special)
            }
            Self::Piece(p, _) => {
                mv.man == ChessMan::from(p.piece)
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
