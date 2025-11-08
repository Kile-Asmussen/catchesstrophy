use std::{
    ops::{Bound, RangeBounds},
    range::{Range, RangeInclusive},
};

use crate::{
    model::{BoardFile, BoardRank, CastlingDirection, ChessColor, ChessMan, DataBoard, Square},
    notation::{
        Parsable,
        fen::{ColorCase, fen_chessman, xtended::CastlingFile},
    },
};

use chumsky::prelude::*;
use chumsky::{IterParser, container::Seq, error::Rich};

/// Generalized Forsyth-Edwards Notation
///
/// The G-FEN is the notation that is the proper superset of
/// FEN, Shredder-FEN, and X-FEN.
///
/// This means:
///
/// - The board can be literally any (rectangular) shape
/// - The chess piece set are extended to include the Princess (A),
///   Empress (C), and Superpieces (X).
/// - Castling can be specified by `KQkq` meaning implicitly the
///   outmost rook has the castling rights or by directly specifying
///   the file of the rook with the rights
/// - En-passant square is optional if en-passant is not possible due
///   to absence of an enemy pawn on an appropriate square
#[derive(Debug, Clone)]
pub struct GFen {
    pub board: Vec<Vec<ChessMan>>,
    pub active_player: ChessColor,
    pub castling: Vec<ColorCase<CastlingFile>>,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl GFen {
    pub fn new(
        board: Vec<Vec<ChessMan>>,
        active_player: ChessColor,
        castling: Vec<ColorCase<CastlingFile>>,
        en_passant: Option<Square>,
        halfmove_clock: u8,
        turn: u16,
    ) -> Self {
        Self {
            board,
            active_player,
            castling,
            en_passant,
            halfmove_clock,
            turn,
        }
    }
}

pub fn gfen_board<'s>(
    files: impl RangeBounds<usize> + 's,
    ranks: impl RangeBounds<usize> + 's,
) -> impl Parser<'s, &'s str, Vec<Vec<Option<ChessMan>>>> {
    let at_least = match ranks.start_bound().copied() {
        Bound::Included(n) => n,
        Bound::Excluded(n) => n.saturating_add(1),
        Bound::Unbounded => 0,
    };

    let at_most = match ranks.end_bound().copied() {
        Bound::Included(n) => n,
        Bound::Excluded(n) => n.saturating_sub(1),
        Bound::Unbounded => usize::MAX,
    };

    gfen_line(files)
        .separated_by(just('/').labelled("solidus (/)"))
        .at_least(at_least)
        .at_most(at_most)
        .collect()
        .boxed()
}

#[test]
fn gfen_board_test() {
    println!("{:?}", gfen_board(3..=100, 2..=2).parse("kQk/qKq"))
}

pub fn gfen_line<'s>(
    range: impl RangeBounds<usize> + 's,
) -> impl Parser<'s, &'s str, Vec<Option<ChessMan>>> {
    let int_range = (Bound::Excluded(0), range.end_bound().copied());
    choice((
        fen_chessman()
            .map(|c| Some(c))
            .repeated()
            .at_least(1)
            .collect(),
        integer(int_range).map(|n| vec![None; n]),
    ))
    .repeated()
    .collect::<Vec<_>>()
    .map(|vv| vv.concat())
    .filter(move |v| range.contains(&v.len()))
    .boxed()
}

#[test]
fn gfen_line_test() {
    println!("{:?}", gfen_line(3..=100).parse("kQk22Kq22"))
}

pub fn integer<'s>(range: impl RangeBounds<usize> + 's) -> impl Parser<'s, &'s str, usize> {
    use Bound::*;
    let label = match (range.start_bound().copied(), range.start_bound().copied()) {
        (Included(lo), Included(hi)) => format!("expected integer >={lo} and <={hi}"),
        (Included(lo), Excluded(hi)) => format!("expected integer >={lo} and <{hi}"),
        (Included(lo), Unbounded) => format!("expected integer >={lo}"),
        (Excluded(lo), Included(hi)) => format!("expected integer >{lo} and <={hi}"),
        (Excluded(lo), Excluded(hi)) => format!("expected integer >{lo} and <{hi}"),
        (Excluded(lo), Unbounded) => format!("expected integer >{lo}"),
        (Unbounded, Included(hi)) => format!("expected integer <={hi}"),
        (Unbounded, Excluded(hi)) => format!("expected integer <{hi}"),
        (Unbounded, Unbounded) => format!("expected integer"),
    };

    chumsky::text::int(10)
        .try_map(|u, _| usize::from_str_radix(u, 10).map_err(|_| EmptyErr::default()))
        .filter(move |u| range.contains(u))
        .boxed()
        .labelled(label)
}

#[test]
pub fn integer_parser() {
    println!("{:?}", integer(0..=10).parse("10"));
    println!("{:?}", integer(0..=10).parse("11"));
}
