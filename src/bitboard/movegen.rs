use std::marker::PhantomData;

use crate::bitboard::{
    attacking::{AttackMaskGenerator, AttackMaskStrategy, Attacks},
    board::BitBoard,
    castling,
    utils::biterate,
    vision::{Panopticon, PawnVision, PieceVision, Vision},
};

use crate::model::*;

pub trait BlessingStrategy {
    type Blessing;
    type Blesser<'a, BB: BitBoard + 'a>: MoveBlesser<'a, BB, BlessedMove = Self::Blessing>;
    fn new<'a, BB: BitBoard>(board: &'a BB) -> Self::Blesser<'a, BB> {
        Self::Blesser::new(board)
    }
}

pub trait MoveBlesser<'a, BB: BitBoard> {
    type BlessedMove;
    fn new(board: &'a BB) -> Self;

    #[inline]
    fn bless(&self, board: &'a BB, mv: ChessMove) -> Option<Self::BlessedMove>;

    #[inline]
    fn bless_into(&self, board: &'a BB, mv: ChessMove, buffer: &mut Vec<Self::BlessedMove>) {
        if let Some(b) = self.bless(board, mv) {
            buffer.push(b)
        }
    }
}

pub struct NoBlessing;

#[repr(transparent)]
pub struct PseudoLegalMoveBlesser<'a, BB: BitBoard>(PhantomData<&'a BB>);

impl BlessingStrategy for NoBlessing {
    type Blessing = PseudoLegal;
    type Blesser<'a, BB: BitBoard + 'a> = PseudoLegalMoveBlesser<'a, BB>;
}

impl<'a, BB: BitBoard> MoveBlesser<'a, BB> for PseudoLegalMoveBlesser<'a, BB> {
    type BlessedMove = PseudoLegal;

    #[inline]
    fn new(board: &'a BB) -> Self {
        Self(PhantomData)
    }

    #[inline]
    fn bless(&self, board: &'a BB, mv: ChessMove) -> Option<Self::BlessedMove> {
        Some(PseudoLegal(mv))
    }
}

pub struct LegalBlessing<AS: AttackMaskStrategy>(PhantomData<AS>);

pub struct LegalMoveBlesser<'a, BB: BitBoard + 'a, AS: AttackMaskStrategy> {
    attack_strat: AS::CachedData<'a, BB>,
    cached_attack: u64,
}

impl<AS: AttackMaskStrategy> BlessingStrategy for LegalBlessing<AS> {
    type Blessing = LegalMove;
    type Blesser<'a, BB: BitBoard + 'a> = LegalMoveBlesser<'a, BB, AS>;
}

impl<'a, BB: BitBoard, AS: AttackMaskStrategy> MoveBlesser<'a, BB>
    for LegalMoveBlesser<'a, BB, AS>
{
    type BlessedMove = LegalMove;

    fn new(board: &'a BB) -> Self {
        let attack_strat = AS::new(board);
        let cached_attack = attack_strat.attacks(board, board.ply().0).attack;
        LegalMoveBlesser {
            attack_strat,
            cached_attack,
        }
    }

    fn bless(&self, board: &'a BB, mv: ChessMove) -> Option<Self::BlessedMove> {
        let player = board.ply().0;
        if self.attack_strat.attacks_after(board, player, mv).check() {
            return None;
        } else if let Some(ix) = CastlingDirection::from_special(mv.special) {
            let castling = board.castling();
            let atks = Attacks {
                attack: self.cached_attack,
                targeted_king: castling.safety[ix.ix()] & castling.back_rank[player.ix()],
            };

            if atks.check() {
                return None;
            }
        }
        Some(LegalMove(mv))
    }
}

pub fn enumerate<'a, BB: BitBoard, X: Panopticon, L: BlessingStrategy>(
    board: &'a BB,
    buffer: &mut Vec<L::Blessing>,
) {
    let total = board.total();
    let pan = X::new(total);
    let blesser = L::new(board);
    let player = board.ply().0;
    let friendly = board.color(player);

    buffer.clear();

    match board.ply().0 {
        ChessColor::WHITE => pawn_moves(
            board,
            &blesser,
            board.men(player, ChessPiece::PAWN),
            pan.white_pawn(),
            buffer,
        ),
        ChessColor::BLACK => pawn_moves(
            board,
            &blesser,
            board.men(player, ChessPiece::PAWN),
            pan.black_pawn(),
            buffer,
        ),
    }

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessPiece::KNIGHT),
        friendly,
        pan.knight(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessPiece::BISHOP),
        friendly,
        pan.bishop(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessPiece::ROOK),
        friendly,
        pan.rook(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessPiece::QUEEN),
        friendly,
        pan.queen(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessPiece::KING),
        friendly,
        pan.king(),
        buffer,
    );

    castling_move(board, &blesser, total, buffer);
}

pub fn pawn_moves<'a, P: PawnVision, BB: BitBoard, L: MoveBlesser<'a, BB>>(
    board: &'a BB,
    blesser: &L,
    pawns: u64,
    pawn_vision: P,
    buffer: &mut Vec<L::BlessedMove>,
) {
    let eps = EnPassant::bit_sq(board.trans().en_passant);
    let enemy = board.color(board.ply().0.opp()) | eps.0;

    biterate! {for from in pawns; {
        biterate! {for to in pawn_vision.push(from); {
            let mut mv = ChessMove {
                from, to,
                ech: ChessPiece::PAWN,
                special: None,
                capture: None
            };

            if from.ix().abs_diff(to.ix()) == 16 {
                mv.special = Some(SpecialMove::PAWN);
            }

            promotions(board, blesser, mv, buffer);
        }}

        biterate! {for to in pawn_vision.hits(from, enemy); {
            let mut mv = ChessMove {
                from, to,
                ech: ChessPiece::PAWN,
                special: None,
                capture: board.commoner_at(to),
            };

            if mv.capture.is_none() {
                mv.special = Some(SpecialMove::PAWN);
                mv.capture = Some(ChessCommoner::PAWN);
            }

            promotions(board, blesser, mv, buffer);
        }}

    }}
}

fn promotions<'a, BB: BitBoard, L: MoveBlesser<'a, BB>>(
    board: &'a BB,
    blesser: &L,
    mut mv: ChessMove,
    buffer: &mut Vec<L::BlessedMove>,
) {
    use SpecialMove::*;
    if mv.to < Square::a1 || Square::h7 < mv.to {
        for spc in [KNIGHT, BISHOP, ROOK, QUEEN] {
            mv.special = Some(spc);

            blesser.bless_into(board, mv, buffer);
        }
    } else {
        blesser.bless_into(board, mv, buffer);
    }
}

fn piece_moves<'a, P: PieceVision, BB: BitBoard, L: MoveBlesser<'a, BB>>(
    board: &'a BB,
    blesser: &L,
    pieces: u64,
    friendly: u64,
    piece: P,
    buffer: &mut Vec<L::BlessedMove>,
) {
    biterate! {for from in pieces; {
        biterate! {for to in piece.hits(from, friendly); {
            let mut mv = ChessMove {
                from, to,
                ech: ChessPiece::from(P::ID),
                special: None,
                capture: board.commoner_at(to),
            };

            blesser.bless_into(board, mv, buffer);
        }}
    }}
}

fn castling_move<'a, BB: BitBoard, L: MoveBlesser<'a, BB>>(
    board: &'a BB,
    blesser: &L,
    total: u64,
    buffer: &mut Vec<L::BlessedMove>,
) {
    use CastlingDirection::*;

    let player = board.ply().0;
    let rights = board.trans().rights;
    let castling = board.castling();

    for dir in [EAST, WEST] {
        if !rights[player.ix()][dir.ix()] {
            continue;
        }

        if (castling.space[dir.ix()] & castling.back_rank[player.ix()] & total) != 0 {
            continue;
        }

        let mv = ChessMove {
            from: castling.rules.king_start[player.ix()],
            to: castling.rules.king_end[player.ix()][dir.ix()],
            ech: ChessPiece::KING,
            special: Some(SpecialMove::from(dir)),
            capture: None,
        };

        blesser.bless_into(board, mv, buffer);
    }
}
