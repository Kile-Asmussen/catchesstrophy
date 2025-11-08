//! # Forsyth-Edwards Notation
//!
//! FEN is the standard way of representing a chess position
//! in standard chess. It consists of six fields separated by whitespace:
//!
//! - The chessboard
//! - The active player
//! - The castling rights
//! - The en-passant square (if applciable)
//! - The half-move clock
//! - The turn number
//!
//! The chessboard is written out as eight solidus-separated (`/`) ranks,
//! starting with the 8th rank, and then in descending order.
//!
//! Each rank has occupied squares denoted by a letter.
//!
//! Empty squares are run-length encoded as digits, 1 meaning a single empty
//! square, 2 meaning two consequtive empty squares, up to 8 meaning an entirely
//! empty rank.
//!
//! The individual ranks are written with files in a-h order.
//!
//! The letters are lower case for black and upper-case for white, P for
//! pawns, N for knights, B for bishops, R for rooks, Q for queens, and K for kings.
//!
//! The active player is written with a single lowercase letter, w for white and b for black.
//!
//! The castling rights are denoted by up to four letters, lowercase for black and uppercase for
//! white, K denotes that kingside castling and Q denotes queenside castling. If all castling rights
//! are forefeit, lost, or used, the field is instead a single dash.
//!
//! The en-passant square is either a square in algebraic coordinate notation, or a single dash
//! of en-passant is not possible.
//!
//! The halfmove clock is a base 10 integer, denoting the number of half-moves (plies) which
//! has elapsed since the last irreversible move (capture or pawn move) and is used in the
//! 50 move rule (a player can unilaterally claim a draw after 100 plies without an irreversible
//! move) and 75 move rule (the game is a forced draw after 150 plies). It starts at 0.
//!
//! The turn number is self-explanatory. It starts at 1.
//!
//! Thus the FEN string of the standard starting position in chess is:
//! ```text
//! rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
//! ```

pub mod generalized;
pub mod shredder;
pub mod xtended;

use chumsky::{prelude::*, text::Char};

use crate::{
    model::*,
    notation::{Parsable, fen::generalized::gfen_board},
};

#[derive(Debug, Clone)]
pub struct FenBoard {
    pub board: DataBoard<Option<ChessMan>>,
    pub to_move: ChessColor,
    pub castling_rights: Vec<ColorCase<CastlingDirection>>,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl FenBoard {
    pub fn new(
        board: DataBoard<Option<ChessMan>>,
        to_move: ChessColor,
        castling_rights: Vec<ColorCase<CastlingDirection>>,
        en_passant: Option<Square>,
        halfmove_clock: u8,
        turn: u16,
    ) -> Self {
        Self {
            board,
            to_move,
            castling_rights,
            en_passant,
            halfmove_clock,
            turn,
        }
    }
}

impl Parsable for FenBoard {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        group((
            fen_board(),
            fen_color(),
            fen_castling(),
            fen_epc_square(),
            fen_halfmove(),
            fen_turn(),
        ))
        .map_group(Self::new)
    }
}

fn fen_board<'s>() -> impl Parser<'s, &'s str, DataBoard<Option<ChessMan>>> {
    gfen_board(8..=8, 8..=8).map(|v| {
        let mut b = DataBoard::new(None);
        b.0.copy_from_slice(&v.concat());
        b
    })
}

fn fen_color<'s>() -> impl Parser<'s, &'s str, ChessColor> {
    choice((
        just('w').to(ChessColor::WHITE),
        just('b').to(ChessColor::BLACK),
    ))
    .labelled("expected w or b")
    .boxed()
}

fn fen_castling<'s>() -> impl Parser<'s, &'s str, Vec<ColorCase<CastlingDirection>>> {
    choice((
        just('-').to(vec![]),
        ColorCase::<CastlingDirection>::parser()
            .repeated()
            .at_least(1)
            .at_most(4)
            .collect(),
    ))
    .boxed()
}

fn fen_epc_square<'s>() -> impl Parser<'s, &'s str, Option<Square>> {
    choice((just('-').to(None), Square::parser().map(|s| Some(s)))).boxed()
}

fn fen_halfmove<'s>() -> impl Parser<'s, &'s str, u8> {
    chumsky::text::int(10)
        .try_map(|i, _| u8::from_str_radix(i, 10).map_err(|_| EmptyErr::default()))
        .labelled("expected integer")
}

fn fen_turn<'s>() -> impl Parser<'s, &'s str, u16> {
    chumsky::text::int(10)
        .try_map(|i, _| u16::from_str_radix(i, 10).map_err(|_| EmptyErr::default()))
        .labelled("expected integer")
}

#[test]
fn fen_board_parsing() {
    println!("{:?}", fen_board().parse("8/8/8/8/8/8/8/8"));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorCase<T> {
    White(T),
    Black(T),
}

impl<T> ColorCase<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> ColorCase<U> {
        match self {
            Self::Black(x) => ColorCase::Black(f(x)),
            Self::White(x) => ColorCase::White(f(x)),
        }
    }
}

impl Parsable for ColorCase<CastlingDirection> {
    fn parser<'s>() -> impl chumsky::Parser<'s, &'s str, Self> {
        use CastlingDirection::*;
        use ColorCase::*;
        choice((
            just('k').to(Black(WEST)),
            just('K').to(White(WEST)),
            just('q').to(Black(EAST)),
            just('Q').to(White(EAST)),
        ))
        .labelled("expected on of K, k, Q, q")
    }
}

pub fn fen_chessman<'s>() -> impl Parser<'s, &'s str, ChessMan> {
    use ChessMan::*;
    choice((
        just('k').to(BLACK_KING),
        just('q').to(BLACK_QUEEN),
        just('r').to(BLACK_ROOK),
        just('b').to(BLACK_BISHOP),
        just('n').to(BLACK_KNIGHT),
        just('p').to(BLACK_PAWN),
        just('P').to(WHITE_PAWN),
        just('N').to(WHITE_KNIGHT),
        just('B').to(WHITE_BISHOP),
        just('R').to(WHITE_ROOK),
        just('Q').to(WHITE_QUEEN),
        just('K').to(WHITE_KING),
    ))
    .labelled("expected one of PNBRQKpnbrqk")
    .boxed()
}
