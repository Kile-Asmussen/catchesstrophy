use crate::model::{
    BitMove, ChessColor, ChessCommoner, ChessEchelon, EnPassant, Legal, PseudoLegal, SpecialMove,
    Square,
    attacks::{Panopticon, PawnVision, Vision},
    bitboard::BitBoard,
    utils::biterate,
};

trait Blesser {
    type Blessed;
    fn new(total: u64) -> Self;

    fn bless<BB: BitBoard>(&self, board: &BB, mv: BitMove) -> Option<Self::Blessed> {
        let mut res = Vec::with_capacity(1);
        self.bless_into(board, mv, &mut res);
        res.pop()
    }

    #[inline]
    fn bless_into<BB: BitBoard>(&self, board: &BB, mv: BitMove, buffer: &mut Vec<Self::Blessed>) {
        if let Some(b) = self.bless(board, mv) {
            buffer.push(b)
        }
    }
}

pub struct NoBlessing;

impl Blesser for NoBlessing {
    type Blessed = PseudoLegal;

    fn new(total: u64) -> Self {
        NoBlessing
    }
    
    #[inline]
    fn bless<BB: BitBoard>(&self, board: &BB, mv: BitMove) -> Option<Self::Blessed> {
        Some(PseudoLegal(mv))
    }
}

pub struct LegalOnly<X: Panopticon>(X);

impl<X: Panopticon> Blesser for LegalOnly<X> {
    type Blessed = Legal

    fn new(total: u64) -> Self {
        LegalOnly(X::new(total))
    }

    fn bless<BB: BitBoard>(&self, board: &BB, mv: BitMove) -> Option<Self::Blessed> {
        todo!()
    }
}

pub fn enumerate<BB: BitBoard, X: Panopticon, L: Blesser>(
    board: &BB,
    buffer: &mut Vec<L::Blessed>,
) {
    let total = board.total();
    let pan = X::new(total);
    let bless = L::new(total);
    let player = board.ply().0;

    match board.ply().0 {
        ChessColor::WHITE => pawn_moves(
            board,
            &bless,
            board.men(player, ChessEchelon::PAWN),
            pan.white_pawn(),
            buffer,
        ),
        ChessColor::BLACK => pawn_moves(
            board,
            &bless,
            board.men(player, ChessEchelon::PAWN),
            pan.black_pawn(),
            buffer,
        ),
    }
}

pub fn pawn_moves<P: PawnVision, BB: BitBoard, L: Blesser>(
    board: &BB,
    blesser: &L,
    pawns: u64,
    pawn_vision: P,
    buffer: &mut Vec<L::Blessed>,
) {
    let eps = EnPassant::bit_sq(board.trans().en_passant);
    let enemy = board.color(board.ply().0.opp()) | eps.0;

    biterate! {for from in pawns; {
        biterate! {for to in pawn_vision.push(from); {
            let mut mv = BitMove {
                from, to,
                ech: ChessEchelon::PAWN,
                special: None,
                capture: None
            };

            if from.ix().abs_diff(to.ix()) == 16 {
                mv.special = Some(SpecialMove::PAWN);
                blesser.bless_into(board, mv, buffer);
            } else {
                promotions(board, blesser, mv, buffer);
            }
        }}

        biterate! {for to in pawn_vision.hits(from, enemy); {
            let mut mv = BitMove {
                from, to,
                ech: ChessEchelon::PAWN,
                special: None,
                capture: None,
            };

            if Some(to) == eps.1 {
                mv.special = Some(SpecialMove::PAWN);
                mv.capture = Some(ChessCommoner::PAWN);
                blesser.bless_into(board, mv, buffer);
            } else {
                promotions(board, blesser, mv, buffer);
            }
        }}

    }}
}

fn promotions<BB: BitBoard, L: Blesser>(board: &BB, blesser: &L, mut mv: BitMove, buffer: &mut Vec<L::Blessed>) {
    use SpecialMove::*;
    if mv.to.ix() < 8 || 55 < mv.to.ix() {
        for spc in [KNIGHT, BISHOP, ROOK, QUEEN] {
            mv.special = Some(spc);

            blesser.bless_into(board, mv, buffer);
        }
    } else {
        blesser.bless_into(board, mv, buffer);
    }
}

// impl BitBoard {
//     pub fn list_pseudomoves<P: Panopticon>(&self, _buffer: &mut Vec<PseudoLegal>) {}

//     pub fn list_moves(&self, _buffer: &mut Vec<Legal>) {}

//     pub fn bless<P: Panopticon>(&self, mv: PseudoLegal) -> Option<Legal> {
//         None
//     }

//     fn attacks<P: Panopticon>(&self, player: Color) -> u64 {
//         let friendly = self.colors[self.player as usize];
//         let enemy = self.colors[self.player.opp() as usize];
//         let total = friendly | enemy;
//         let pan = P::new(total);

//         let king = self.men[ChessMan::KING as usize - 1] & friendly;
//         let superpiece_mask = pan.queen().surveil(king) | pan.knight().surveil(king);
//         let superpiece_mask = enemy & superpiece_mask;
//         let mut threats = 0;

//         let pawn = (self.men[ChessMan::PAWN as usize - 1] & superpiece_mask);
//         threats |= pan.white_pawn().surveil(pawn);

//         let knight = (self.men[ChessMan::PAWN as usize - 1] & superpiece_mask);
//         threats |= pan.knight().surveil(knight);

//         let bishop = (self.men[ChessMan::BISHOP as usize - 1] & superpiece_mask);
//         threats |= pan.bishop().surveil(bishop);

//         let rook = (self.men[ChessMan::ROOK as usize - 1] & superpiece_mask);
//         threats |= pan.rook().surveil(rook);

//         let queen = (self.men[ChessMan::QUEEN as usize - 1] & superpiece_mask);
//         threats |= pan.queen().surveil(queen);

//         return threats;
//     }
// }

// trait BitBoardAttackStrat {
//     fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64;
//     fn counterattacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64;
// }

// #[repr(transparent)]
// struct CopyMakeAttacks<'a>(&'a BitBoard);

// impl<'a> BitBoardAttackStrat for CopyMakeAttacks<'a> {
//     fn counterattacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
//         let mut board = self.0.clone();
//         let bits = (1 << mv.from as u8) ^ (1 << mv.to as u128);
//         board.fake_move(mv);
//         board.attacks::<P>(player.opp())
//     }

//     fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
//         let mut board = self.0.clone();
//         let bits = (1 << mv.from as u8) ^ (1 << mv.to as u128);
//         board.fake_move(mv);
//         board.attacks::<P>(player)
//     }
// }

// #[repr(transparent)]
// struct MakeUnmakeAttacks<'a>(&'a BitBoard);

// impl<'a> BitBoardAttackStrat for MakeUnmakeAttacks<'a> {
//     fn attacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
//         0
//     }

//     fn counterattacks_after_move<P: Panopticon>(self, player: Color, mv: BitMove) -> u64 {
//         0
//     }
// }
