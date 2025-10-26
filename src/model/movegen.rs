use crate::model::{
    BitBoard, BitMove, Color, Legal, Piece, PseudoLegal,
    attacks::{Panopticon, Vision},
};

impl BitBoard {
    pub fn list_pseudomoves<P: Panopticon>(&self, _buffer: &mut Vec<PseudoLegal>) {}

    pub fn list_moves(&self, _buffer: &mut Vec<Legal>) {}

    pub fn bless<P: Panopticon>(&self, mv: PseudoLegal) -> Option<Legal> {
        None
    }

    fn attacks<P: Panopticon>(&self, player: Color) -> u64 {
        let opponent = self.player.opp();
        let friendly = self.colors[self.player as usize];
        let enemy = self.colors[opponent as usize];
        let total = friendly | enemy;
        let pan = P::new(total);

        let king = self.pieces[Piece::KING as usize - 1] & friendly;
        let superpiece_mask = pan.queen().surveil(king) | pan.knight().surveil(king);
        let superpiece_mask = enemy & superpiece_mask;
        let mut threats = 0;

        let pawn = (self.pieces[Piece::PAWN as usize - 1] & superpiece_mask);
        threats |= pan.pawn(opponent).surveil(pawn);

        let knight = (self.pieces[Piece::PAWN as usize - 1] & superpiece_mask);
        threats |= pan.knight().surveil(knight);

        let bishop = (self.pieces[Piece::BISHOP as usize - 1] & superpiece_mask);
        threats |= pan.bishop().surveil(bishop);

        let rook = (self.pieces[Piece::ROOK as usize - 1] & superpiece_mask);
        threats |= pan.rook().surveil(rook);

        let queen = (self.pieces[Piece::QUEEN as usize - 1] & superpiece_mask);
        threats |= pan.queen().surveil(queen);

        return threats;
    }
}

trait BitBoardAttackStrat {
    fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64;
    fn counterattacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64;
}

#[repr(transparent)]
struct CopyMakeAttacks<'a>(&'a BitBoard);

impl<'a> BitBoardAttackStrat for CopyMakeAttacks<'a> {
    fn counterattacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
        let mut board = self.0.clone();
        let bits = (1 << mv.from as u8) ^ (1 << mv.to as u128);

        board.pieces[(mv.piece as usize).saturating_mul(1)] ^= bits;
        board.colors[player as usize] ^= bits;

        if !mv.capture.is_none() {
            let cap_bit = 1 << mv.attack as u8;
            board.pieces[(mv.capture as usize).saturating_sub(1)] ^= cap_bit;
            board.colors[player.opp() as usize] ^= cap_bit;
        }

        board.attacks::<P>(player.opp())
    }

    fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
        todo!()
    }
}

#[repr(transparent)]
struct MakeUnmakeAttacks<'a>(&'a BitBoard);

impl<'a> BitBoardAttackStrat for MakeUnmakeAttacks<'a> {
    fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
        todo!()
    }

    fn counterattacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
        let bits = (1 << mv.from as u8) ^ (1 << mv.to as u8);
        let opponent = player.opp();
        let friendly = self.0.colors[player as usize] ^ bits;
        let enemy = self.0.colors[opponent as usize];
        let total = friendly | enemy;
        let pan = P::new(total);

        let king = self.0.pieces[Piece::KING as usize - 1] & friendly;
        let superpiece_mask = pan.queen().surveil(king) | pan.knight().surveil(king);
        let superpiece_mask = enemy & superpiece_mask;
        let mut threats = 0;

        let cap = 1 << mv.attack as u8;

        let pawn = (self.0.pieces[Piece::PAWN as usize - 1] & superpiece_mask)
            ^ is(Piece::PAWN, mv.capture, cap);
        threats |= pan.pawn(opponent).surveil(pawn);

        let knight = (self.0.pieces[Piece::PAWN as usize - 1] & superpiece_mask)
            ^ is(Piece::KNIGHT, mv.capture, cap);
        threats |= pan.knight().surveil(knight);

        let bishop = (self.0.pieces[Piece::BISHOP as usize - 1] & superpiece_mask)
            ^ is(Piece::BISHOP, mv.capture, cap);
        threats |= pan.bishop().surveil(bishop);

        let rook = (self.0.pieces[Piece::ROOK as usize - 1] & superpiece_mask)
            ^ is(Piece::ROOK, mv.capture, cap);
        threats |= pan.rook().surveil(rook);

        let queen = (self.0.pieces[Piece::QUEEN as usize - 1] & superpiece_mask)
            ^ is(Piece::QUEEN, mv.capture, cap);
        threats |= pan.queen().surveil(queen);

        return threats;

        #[inline]
        fn is(piece1: Piece, piece2: Piece, bits: u64) -> u64 {
            if piece1 == piece1 { bits } else { 0 }
        }
    }
}
