use crate::model::{
    BitBoard, BitMove, Castles, Color, Legal, Piece, Rights, Special, Square, TransientInfo,
    ZOBHASHER, ZobHasher,
    notation::{AlgNotaion, CoordNotation},
};

impl BitBoard {
    pub fn rehash(&self) -> u64 {
        use Color::*;
        use Piece::*;

        let mut res = 0;

        let zobhasher = &*ZOBHASHER;

        for piece in [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING] {
            let board = &zobhasher.pieces[piece as usize];
            res ^= zobhasher.hash_mask(board, self.pieces[piece as usize]);
        }

        for color in [WHITE, BLACK] {
            let board = &zobhasher.colors[color as usize];
            res ^= zobhasher.hash_mask(board, self.colors[color as usize]);
        }

        res ^= zobhasher.hash_rights(self.trans.rights);

        if let Some(eq_square) = self.trans.ep_square {
            res ^= zobhasher.hash_file(eq_square as u8);
        }

        if self.player == BLACK {
            res ^= zobhasher.black_to_move;
        }

        res
    }

    pub fn make_move(&mut self, mv: Legal) -> TransientInfo {
        let mv = mv.0;
        let res = self.trans;
        let zobhasher = &*ZOBHASHER;

        self.simple_move(mv, zobhasher);
        self.promotion_move(mv, zobhasher);
        self.castling_move(mv, zobhasher);

        self.update_transient(mv, zobhasher);

        self.hash ^= zobhasher.black_to_move;
        self.turn += self.player as u16;
        self.player = self.player.opp();

        res
    }

    pub fn unmake_move(&mut self, mv: Legal, trans: TransientInfo) {
        let mv = mv.0;
        let zobhasher = &*ZOBHASHER;

        self.simple_move(mv, zobhasher);
        self.promotion_move(mv, zobhasher);
        self.castling_move(mv, zobhasher);

        self.trans = trans;
        self.update_transient(mv, zobhasher);
        self.trans = trans;

        self.hash ^= zobhasher.black_to_move;
        self.player = self.player.opp();
        self.turn -= self.player as u16;
    }

    #[inline]
    fn simple_move(&mut self, mv: BitMove, zobhasher: &ZobHasher) {
        if Special::EAST <= mv.special {
            return;
        }

        let piece = (mv.piece as usize).saturating_sub(1);
        let bits = (1 << mv.from as u8) | (1 << mv.to as u8);
        let is_cap = !mv.capture.is_none();
        let cap_piece = (mv.capture as usize).saturating_sub(1);
        let cap_bit = 1 << mv.attack as u8;
        let cap_sq = mv.attack as usize;
        let player = self.player as usize;
        let opponent = self.player.opp() as usize;

        self.pieces[piece] ^= bits;
        self.colors[player] ^= bits;
        self.colors[opponent] ^= cap_bit;

        self.hash ^= zobhasher.hash_mask(&zobhasher.pieces[piece], bits);
        self.hash ^= zobhasher.hash_mask(&zobhasher.colors[player], bits);

        if is_cap {
            self.pieces[cap_piece] ^= cap_bit;
            self.hash ^= zobhasher.pieces[cap_piece][cap_sq];
            self.hash ^= zobhasher.colors[opponent][cap_sq];
        }
    }

    #[inline]
    fn promotion_move(&mut self, mv: BitMove, zobhasher: &ZobHasher) {
        if mv.special < Special::KNIGHT || Special::QUEEN < mv.special {
            return;
        }

        let pawn = Piece::PAWN as usize;
        let piece = (mv.piece as usize).saturating_sub(1);
        let bit = 1 << mv.to as u8;
        let to = mv.to as usize;

        self.pieces[pawn] ^= bit;
        self.pieces[piece] ^= bit;

        self.hash ^= zobhasher.pieces[pawn][to];
        self.hash ^= zobhasher.pieces[piece][to];
    }

    #[inline]
    fn castling_move(&mut self, mv: BitMove, zobhasher: &ZobHasher) {
        if mv.special < Special::EAST {
            return;
        }

        let dir = mv.special as usize - Special::EAST as usize;
        let offset = if self.player.is_black() { 56 } else { 0 };
        let rank = 0xFF << offset;
        let king = Piece::KING as usize - 1;
        let king_move = self.castling.king_move[dir] & rank;
        let rook = Piece::ROOK as usize - 1;
        let rook_move = self.castling.rook_move[dir] & rank;
        let player = self.player as usize;

        self.pieces[king] ^= king_move;
        self.pieces[rook] ^= rook_move;
        self.colors[player] ^= king_move;
        self.colors[player] ^= rook_move;

        self.hash ^= zobhasher.hash_mask(&zobhasher.pieces[king], king_move);
        self.hash ^= zobhasher.hash_mask(&zobhasher.pieces[rook], rook_move);
        self.hash ^= zobhasher.hash_mask(&zobhasher.colors[player], king_move | rook_move);
    }

    #[inline]
    fn update_transient(&mut self, mv: BitMove, zobhasher: &ZobHasher) {
        let player = self.player as usize;
        let opponent = self.player.opp() as usize;
        let ix = player << 1;

        self.hash ^= zobhasher.hash_rights(self.trans.rights);

        if mv.piece == Piece::KING {
            let bits = 0x3 << ix;
            self.trans.rights.0 &= !bits;
        }

        for (piece, square, color) in
            [(mv.piece, mv.from, player), (mv.capture, mv.attack, opponent)]
        {
            if piece == Piece::ROOK {
                for dir in [Castles::EAST, Castles::WEST] {
                    let dir = dir as usize;
                    if square == self.castling.rook_from[dir] {
                        let ix = color << 1 | dir;
                        let bit = 1 << ix;
                        if self.trans.rights.0 & bit != 0 {
                            self.hash ^= zobhasher.castling[ix];
                        }
                        self.trans.rights.0 &= !bit;
                    }
                }
            }
        }

        self.hash ^= zobhasher.hash_rights(self.trans.rights);

        if mv.capture != Piece::NONE || mv.piece == Piece::PAWN {
            self.trans.halfmove_clock = 0;
        } else {
            self.trans.halfmove_clock += 1;
        }

        if let Some(ep_square) = self.trans.ep_square {
            self.hash ^= zobhasher.hash_file(ep_square as u8);
        }

        if mv.piece == Piece::PAWN && (mv.from as u8).abs_diff(mv.to as u8) == 16 {
            let ep_ix = (mv.from as u8).min(mv.to as u8) + 8;
            self.trans.ep_square = Square::from_repr(ep_ix);
            self.hash ^= zobhasher.hash_file(ep_ix);
        } else {
            self.trans.ep_square = None;
        }
    }
}

impl ZobHasher {
    #[inline]
    pub fn hash_mask(&self, board: &[u64; 64], mut mask: u64) -> u64 {
        let mut res = 0;
        for _ in 0..mask.count_ones() {
            let sq = mask.trailing_zeros();
            let bit = 1 << sq;
            mask ^= bit;
            res ^= board[sq as usize];
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
    fn matches(mv: BitMove) -> bool;
}

impl MoveMatcher for CoordNotation {
    fn matches(_mv: BitMove) -> bool {
        todo!()
    }
}

impl MoveMatcher for AlgNotaion {
    fn matches(_mv: BitMove) -> bool {
        todo!()
    }
}
