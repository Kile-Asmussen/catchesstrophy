use std::marker::PhantomData;

use crate::model::{
    BitMove, CastlingDirection, ChessColor, ChessCommoner, ChessEchelon, ChessPiece, EnPassant,
    LegalMove, PseudoLegal, SpecialMove, Square,
    attacking::{AttackMaskGenerator, AttackMaskStrategy, Attacks},
    bitboard::BitBoard,
    castling,
    notation::show_mask,
    utils::biterate,
    vision::{Panopticon, PawnVision, PieceVision, Vision},
};

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
    fn bless(&self, board: &'a BB, mv: BitMove) -> Option<Self::BlessedMove>;

    #[inline]
    fn bless_into(&self, board: &'a BB, mv: BitMove, buffer: &mut Vec<Self::BlessedMove>) {
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

    fn new(board: &'a BB) -> Self {
        Self(PhantomData)
    }

    fn bless(&self, board: &'a BB, mv: BitMove) -> Option<Self::BlessedMove> {
        Some(PseudoLegal(mv))
    }
}

pub struct LegalBlessing<AS: AttackMaskStrategy, X: Panopticon>(PhantomData<(AS, X)>);

pub struct LegalMoveBlesser<'a, BB: BitBoard + 'a, AS: AttackMaskStrategy, X: Panopticon> {
    attack_strat: AS::CachedMasks<'a, BB>,
    cached_attack: u64,
    _x: PhantomData<X>,
}

impl<AS: AttackMaskStrategy, X: Panopticon> BlessingStrategy for LegalBlessing<AS, X> {
    type Blessing = LegalMove;
    type Blesser<'a, BB: BitBoard + 'a> = LegalMoveBlesser<'a, BB, AS, X>;
}

impl<'a, BB: BitBoard, AS: AttackMaskStrategy, X: Panopticon> MoveBlesser<'a, BB>
    for LegalMoveBlesser<'a, BB, AS, X>
{
    type BlessedMove = LegalMove;

    fn new(board: &'a BB) -> Self {
        let attack_strat = AS::new(board);
        let cached_attack = attack_strat.attacks::<X>(board, board.ply().0).attack;
        LegalMoveBlesser {
            attack_strat,
            cached_attack,
            _x: PhantomData,
        }
    }

    fn bless(&self, board: &'a BB, mv: BitMove) -> Option<Self::BlessedMove> {
        let player = board.ply().0;
        if self
            .attack_strat
            .attacks_after::<X>(board, player, mv)
            .check()
        {
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

    match board.ply().0 {
        ChessColor::WHITE => pawn_moves(
            board,
            &blesser,
            board.men(player, ChessEchelon::PAWN),
            pan.white_pawn(),
            buffer,
        ),
        ChessColor::BLACK => pawn_moves(
            board,
            &blesser,
            board.men(player, ChessEchelon::PAWN),
            pan.black_pawn(),
            buffer,
        ),
    }

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessEchelon::KNIGHT),
        friendly,
        pan.knight(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessEchelon::BISHOP),
        friendly,
        pan.bishop(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessEchelon::ROOK),
        friendly,
        pan.rook(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessEchelon::QUEEN),
        friendly,
        pan.queen(),
        buffer,
    );

    piece_moves(
        board,
        &blesser,
        board.men(player, ChessEchelon::KING),
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
            let mut mv = BitMove {
                from, to,
                ech: ChessEchelon::PAWN,
                special: None,
                capture: None
            };

            if from.ix().abs_diff(to.ix()) == 16 {
                mv.special = Some(SpecialMove::PAWN);
            }

            promotions(board, blesser, mv, buffer);
        }}

        biterate! {for to in pawn_vision.hits(from, enemy); {
            let mut mv = BitMove {
                from, to,
                ech: ChessEchelon::PAWN,
                special: None,
                capture: board.comm_at(to),
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
    mut mv: BitMove,
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
            let mut mv = BitMove {
                from, to,
                ech: ChessEchelon::from(P::ID),
                special: None,
                capture: board.comm_at(to),
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

        let mv = BitMove {
            from: castling.king_start[player.ix()],
            to: castling.king_end[player.ix()][dir.ix()],
            ech: ChessEchelon::KING,
            special: Some(SpecialMove::from(dir)),
            capture: None,
        };

        blesser.bless_into(board, mv, buffer);
    }
}
