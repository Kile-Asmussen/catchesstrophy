use crate::model::{
    BitMove, ChessMan, Color, Legal, PseudoLegal,
    attacks::{Panopticon, Vision},
};

impl BitBoard {
    pub fn list_pseudomoves<P: Panopticon>(&self, _buffer: &mut Vec<PseudoLegal>) {}

    pub fn list_moves(&self, _buffer: &mut Vec<Legal>) {}

    pub fn bless<P: Panopticon>(&self, mv: PseudoLegal) -> Option<Legal> {
        None
    }

    fn attacks<P: Panopticon>(&self, player: Color) -> u64 {
        let friendly = self.colors[self.player as usize];
        let enemy = self.colors[self.player.opp() as usize];
        let total = friendly | enemy;
        let pan = P::new(total);

        let king = self.men[ChessMan::KING as usize - 1] & friendly;
        let superpiece_mask = pan.queen().surveil(king) | pan.knight().surveil(king);
        let superpiece_mask = enemy & superpiece_mask;
        let mut threats = 0;

        let pawn = (self.men[ChessMan::PAWN as usize - 1] & superpiece_mask);
        threats |= pan.white_pawn().surveil(pawn);

        let knight = (self.men[ChessMan::PAWN as usize - 1] & superpiece_mask);
        threats |= pan.knight().surveil(knight);

        let bishop = (self.men[ChessMan::BISHOP as usize - 1] & superpiece_mask);
        threats |= pan.bishop().surveil(bishop);

        let rook = (self.men[ChessMan::ROOK as usize - 1] & superpiece_mask);
        threats |= pan.rook().surveil(rook);

        let queen = (self.men[ChessMan::QUEEN as usize - 1] & superpiece_mask);
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
        board.fake_move(mv);
        board.attacks::<P>(player.opp())
    }

    fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
        let mut board = self.0.clone();
        let bits = (1 << mv.from as u8) ^ (1 << mv.to as u128);
        board.fake_move(mv);
        board.attacks::<P>(player)
    }
}

#[repr(transparent)]
struct MakeUnmakeAttacks<'a>(&'a BitBoard);

impl<'a> BitBoardAttackStrat for MakeUnmakeAttacks<'a> {
    fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
        0
    }

    fn counterattacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
        0
    }
}
