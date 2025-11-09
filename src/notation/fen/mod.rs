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

use std::collections::HashSet;

use chumsky::{prelude::*, text::Char};

use crate::{
    model::*,
    notation::{
        Parsable, Prs,
        fen::generalized::{gfen_8x8_board, gfen_board, gfen_castling},
    },
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

    pub fn sanity_check(&self) -> Result<(), String> {
        if self.castling_rights.len()
            != self
                .castling_rights
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len()
        {
            Err("duplicate in castling rights field")?;
        }

        self.castling_check(ColorCase::White(CastlingDirection::EAST))?;
        self.castling_check(ColorCase::Black(CastlingDirection::EAST))?;
        self.castling_check(ColorCase::White(CastlingDirection::WEST))?;
        self.castling_check(ColorCase::Black(CastlingDirection::WEST))?;

        self.epc_check()?;

        return Ok(());
    }

    fn epc_check(&self) -> Result<(), String> {
        if let Some(sq) = self.en_passant {
            match (self.to_move, sq.coords().1) {
                (ChessColor::WHITE, BoardRank::_3) => return Ok(()),
                (ChessColor::BLACK, BoardRank::_6) => return Ok(()),
                _ => {}
            }
        }

        Err("illegal en-passant square".to_string())
    }

    fn castling_check(&self, c: ColorCase<CastlingDirection>) -> Result<(), String> {
        use CastlingDirection::*;
        use ChessMan::*;
        use ColorCase::*;
        let (k, ksq, r, rsq, col, side) = match c {
            White(EAST) => (
                WHITE_KING,
                Square::e1,
                WHITE_ROOK,
                Square::a1,
                "white",
                "queenside",
            ),
            Black(EAST) => (
                BLACK_KING,
                Square::e8,
                BLACK_ROOK,
                Square::a8,
                "black",
                "queenside",
            ),
            White(WEST) => (
                WHITE_KING,
                Square::e1,
                WHITE_ROOK,
                Square::h1,
                "white",
                "kingside",
            ),
            Black(WEST) => (
                BLACK_KING,
                Square::e8,
                BLACK_ROOK,
                Square::h8,
                "black",
                "kingside",
            ),
        };

        if self.board.get(ksq) != &Some(k) || self.board.get(rsq) != &Some(r) {
            Err(format!("{col} cannot castle {side}"))
        } else {
            Ok(())
        }
    }
}

impl Parsable for FenBoard {
    fn parser<'s>() -> impl Prs<'s, Self> {
        group((
            fen_board().then_ignore(ws()),
            fen_color().then_ignore(ws()),
            gfen_castling().then_ignore(ws()),
            fen_epc_square().then_ignore(ws()),
            fen_halfmove().then_ignore(ws()),
            fen_turn(),
        ))
        .map_group(Self::new)
        .boxed()
    }
}

fn ws<'s>() -> impl Prs<'s, ()> {
    chumsky::text::whitespace().at_least(1)
}

fn fen_board<'s>() -> impl Prs<'s, DataBoard<Option<ChessMan>>> {
    gfen_8x8_board(fen_chessman())
}

fn fen_color<'s>() -> impl Prs<'s, ChessColor> {
    choice((
        just('w').to(ChessColor::WHITE),
        just('b').to(ChessColor::BLACK),
    ))
    .labelled("w or b")
    .boxed()
}

fn fen_epc_square<'s>() -> impl Prs<'s, Option<Square>> {
    choice((just('-').to(None), Square::parser().map(|s| Some(s)))).boxed()
}

fn fen_halfmove<'s>() -> impl Prs<'s, u8> {
    chumsky::text::int(10)
        .try_map(|i, span| {
            u8::from_str_radix(i, 10)
                .map_err(|_| Rich::custom(span, format!("unable to parse {i} as u8")))
        })
        .labelled("integer")
        .boxed()
}

fn fen_turn<'s>() -> impl Prs<'s, u16> {
    chumsky::text::int(10)
        .try_map(|i, span| {
            u16::from_str_radix(i, 10)
                .map_err(|_| Rich::custom(span, format!("unable to parse {i} as u16")))
        })
        .labelled("integer")
        .boxed()
}

#[test]
fn fen_board_parsing() {
    println!("{:?}", fen_board().parse("8/8/8/8/8/8/8/8"));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    fn parser<'s>() -> impl Prs<'s, Self> {
        use CastlingDirection::*;
        use ColorCase::*;
        choice((
            just('k').to(Black(WEST)),
            just('K').to(White(WEST)),
            just('q').to(Black(EAST)),
            just('Q').to(White(EAST)),
        ))
        .labelled("one of K, k, Q, q")
        .boxed()
    }
}

pub fn fen_chessman<'s>() -> impl Prs<'s, ChessMan> {
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
    .labelled("one of PNBRQKpnbrqk")
    .boxed()
}
