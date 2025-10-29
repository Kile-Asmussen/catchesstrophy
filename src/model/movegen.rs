use std::marker::PhantomData;

use crate::model::{
    BitMove, CastlingDirection, ChessColor, ChessCommoner, ChessEchelon, ChessPiece, EnPassant,
    LegalMove, PseudoLegal, SpecialMove, Square,
    attacking::{AttackMaskStrategy, Attacks},
    bitboard::BitBoard,
    castling,
    notation::show_mask,
    utils::biterate,
    vision::{Panopticon, PawnVision, PieceVision, Vision},
};

pub trait Blesser<'a> {
    type Blessed;
    fn new<BB: BitBoard>(board: &'a BB) -> Self;

    #[inline]
    fn bless<BB: BitBoard>(&self, board: &'a BB, mv: BitMove) -> Option<Self::Blessed>;

    #[inline]
    fn bless_into<BB: BitBoard>(
        &self,
        board: &'a BB,
        mv: BitMove,
        buffer: &mut Vec<Self::Blessed>,
    ) {
        if let Some(b) = self.bless(board, mv) {
            buffer.push(b)
        }
    }
}

pub struct NoBlessing;

impl<'a> Blesser<'a> for NoBlessing {
    type Blessed = PseudoLegal;

    fn new<BB: BitBoard>(_: &BB) -> Self {
        NoBlessing
    }

    #[inline]
    fn bless<BB: BitBoard>(&self, board: &BB, mv: BitMove) -> Option<Self::Blessed> {
        Some(PseudoLegal(mv))
    }
}

pub struct LegalBlessing<'a, AS: AttackMaskStrategy<'a>, X: Panopticon> {
    attack_strat: AS,
    cached_attack: u64,
    _lt: PhantomData<&'a X>,
}

impl<'a, A: AttackMaskStrategy<'a>, X: Panopticon> Blesser<'a> for LegalBlessing<'a, A, X> {
    type Blessed = LegalMove;

    fn new<BB: BitBoard>(board: &'a BB) -> Self {
        let attack_strat = A::new(board);
        let cached_attack = attack_strat.attacks::<BB, X>(board, board.ply().0).attack;
        Self {
            attack_strat,
            cached_attack,
            _lt: PhantomData,
        }
    }

    fn bless<BB: BitBoard>(&self, board: &'a BB, mv: BitMove) -> Option<Self::Blessed> {
        let player = board.ply().0;
        if self
            .attack_strat
            .attacks_after::<BB, X>(board, player, mv)
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

        return Some(LegalMove(mv));
    }
}

pub fn enumerate<'a, BB: BitBoard, X: Panopticon, L: Blesser<'a>>(
    board: &'a BB,
    buffer: &mut Vec<L::Blessed>,
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

pub fn pawn_moves<'a, P: PawnVision, BB: BitBoard, L: Blesser<'a>>(
    board: &'a BB,
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

fn promotions<'a, BB: BitBoard, L: Blesser<'a>>(
    board: &'a BB,
    blesser: &L,
    mut mv: BitMove,
    buffer: &mut Vec<L::Blessed>,
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

fn piece_moves<'a, P: PieceVision, BB: BitBoard, L: Blesser<'a>>(
    board: &'a BB,
    blesser: &L,
    pieces: u64,
    friendly: u64,
    piece: P,
    buffer: &mut Vec<L::Blessed>,
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

fn castling_move<'a, BB: BitBoard, L: Blesser<'a>>(
    board: &'a BB,
    blesser: &L,
    total: u64,
    buffer: &mut Vec<L::Blessed>,
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
