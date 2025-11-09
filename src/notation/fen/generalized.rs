//! Generalized Forsyth-Edwards Notation
//!
//! The G-FEN is the notation that is the proper superset of
//! FEN, Shredder-FEN, and X-FEN.
//!
//! This means:
//!
//! - The board can be literally any (rectangular) shape
//! - The chess piece set are extended to include the Princess (A),
//!   Empress (C).
//! - Castling can be specified by `KQkq` meaning implicitly the
//!   outmost rook has the castling rights or by directly specifying
//!   the file of the rook with the rights (preferred)
//! - En-passant square is optional if en-passant is not possible due
//!   to absence of an enemy pawn on an appropriate square

use std::{
    ops::{Bound, RangeBounds},
    range::{Range, RangeInclusive},
};

use crate::{
    model::{BoardFile, BoardRank, CastlingDirection, ChessColor, ChessMan, DataBoard, Square},
    notation::{
        Parsable, Prs,
        fen::{
            ColorCase, fen_board, fen_chessman, fen_color, fen_epc_square, fen_halfmove, fen_turn,
            ws, xtended::CastlingFile,
        },
    },
};

use chumsky::prelude::*;
use chumsky::{IterParser, container::Seq, error::Rich};

#[derive(Debug, Clone)]
pub struct StdGenFenBoard {
    pub board: DataBoard<Option<ChessMan>>,
    pub active_player: ChessColor,
    pub castling: Vec<ColorCase<CastlingFile>>,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl StdGenFenBoard {
    pub fn new(
        board: DataBoard<Option<ChessMan>>,
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

impl Parsable for StdGenFenBoard {
    fn parser<'s>() -> impl Prs<'s, Self> {
        group((
            fen_board().then_ignore(ws()),
            fen_color().then_ignore(ws()),
            gfen_castling().then_ignore(ws()),
            fen_epc_square().then_ignore(ws()),
            fen_halfmove().then_ignore(ws()),
            fen_turn(),
        ))
        .padded()
        .map_group(Self::new)
        .boxed()
    }
}

#[derive(Debug, Clone)]
pub struct GenFenBoard<Man>
where
    Man: Clone + Copy + Parsable + 'static,
{
    pub board: Vec<Vec<Option<Man>>>,
    pub active_player: ChessColor,
    pub castling: Vec<ColorCase<char>>,
    pub en_passant: Option<(char, u8)>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl<Man> GenFenBoard<Man>
where
    Man: Clone + Copy + Parsable + 'static,
{
    pub fn new(
        board: Vec<Vec<Option<Man>>>,
        active_player: ChessColor,
        castling: Vec<ColorCase<char>>,
        en_passant: Option<(char, u8)>,
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

impl<Man> Parsable for GenFenBoard<Man>
where
    Man: Clone + Copy + Parsable + 'static,
{
    fn parser<'s>() -> impl Prs<'s, Self> {
        group((
            gfen_board(1..=26, 1..=100, Man::parser()).then_ignore(ws()),
            fen_color().then_ignore(ws()),
            gfen_castling().then_ignore(ws()),
            gfen_epc_square().then_ignore(ws()),
            fen_halfmove().then_ignore(ws()),
            fen_turn(),
        ))
        .map_group(Self::new)
        .boxed()
    }
}

impl Parsable for ColorCase<char> {
    fn parser<'s>() -> impl Prs<'s, Self> {
        choice((
            one_of('a'..='z').map(Self::Black),
            one_of('A'..='Z').map(Self::Black),
        ))
        .boxed()
    }
}

pub(crate) fn gfen_epc_square<'s>() -> impl Prs<'s, Option<(char, u8)>> {
    choice((
        one_of('a'..='z')
            .then(parse_usize(0..=100).map(|u| u as u8))
            .map(Some),
        just('-').to(None),
    ))
    .boxed()
}

pub(crate) fn gfen_castling<'s, CastlingThing>() -> impl Prs<'s, Vec<ColorCase<CastlingThing>>>
where
    CastlingThing: Clone + 's,
    ColorCase<CastlingThing>: Parsable,
{
    choice((
        just('-').to(vec![]),
        ColorCase::<CastlingThing>::parser()
            .repeated()
            .at_least(1)
            .at_most(4)
            .collect(),
    ))
    .boxed()
}

pub(crate) fn gfen_8x8_board<'s, ChessThing>(
    man: impl Prs<'s, ChessThing> + 's,
) -> impl Prs<'s, DataBoard<Option<ChessThing>>>
where
    ChessThing: Clone + 's,
{
    gfen_board(8..=8, 8..=8, man)
        .map(|v| {
            let mut b = DataBoard::new(|| None);
            b.0.clone_from_slice(&v.concat());
            b
        })
        .boxed()
}

pub(crate) fn gfen_board<'s, ChessThing>(
    files: impl RangeBounds<usize> + 's,
    ranks: impl RangeBounds<usize> + 's,
    man: impl Prs<'s, ChessThing> + 's,
) -> impl Prs<'s, Vec<Vec<Option<ChessThing>>>>
where
    ChessThing: Clone + 's,
{
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

    gfen_line(files, man)
        .separated_by(just('/').labelled("solidus (/)"))
        .at_least(at_least)
        .at_most(at_most)
        .collect()
        .boxed()
}

#[test]
fn gfen_board_test() {
    println!(
        "{:?}",
        gfen_board(3..=100, 2..=2, fen_chessman()).parse("kQk/qKq")
    )
}

pub(crate) fn gfen_line<'s, ChessThing>(
    range: impl RangeBounds<usize> + 's,
    man: impl Prs<'s, ChessThing> + 's,
) -> impl Prs<'s, Vec<Option<ChessThing>>>
where
    ChessThing: Clone + 's,
{
    let int_range = (Bound::Excluded(0), range.end_bound().copied());
    choice((
        man.map(|c| Some(c)).repeated().at_least(1).collect(),
        parse_usize(int_range).map(|n| vec![None; n]),
    ))
    .repeated()
    .collect::<Vec<_>>()
    .map(|vv| vv.concat())
    .filter(move |v| range.contains(&v.len()))
    .boxed()
}

#[test]
fn gfen_line_test() {
    println!(
        "{:?}",
        gfen_line(3..=100, fen_chessman()).parse("kQk22Kq22")
    )
}

pub(crate) fn parse_usize<'s>(range: impl RangeBounds<usize> + 's) -> impl Prs<'s, usize> {
    use Bound::*;
    let label = match (range.start_bound().copied(), range.end_bound().copied()) {
        (Included(lo), Included(hi)) => format!("integer >={lo} and <={hi}"),
        (Included(lo), Excluded(hi)) => format!("integer >={lo} and <{hi}"),
        (Included(lo), Unbounded) => format!("integer >={lo}"),
        (Excluded(lo), Included(hi)) => format!("integer >{lo} and <={hi}"),
        (Excluded(lo), Excluded(hi)) => format!("integer >{lo} and <{hi}"),
        (Excluded(lo), Unbounded) => format!("integer >{lo}"),
        (Unbounded, Included(hi)) => format!("integer <={hi}"),
        (Unbounded, Excluded(hi)) => format!("integer <{hi}"),
        (Unbounded, Unbounded) => format!("integer"),
    };

    chumsky::text::int(10)
        .try_map(|u, span| {
            usize::from_str_radix(u, 10)
                .map_err(|_| Rich::custom(span, format!("could not parse {u} as usize")))
        })
        .filter(move |u| range.contains(u))
        .labelled(label)
        .boxed()
}

#[test]
pub fn integer_parser() {
    println!("{:?}", parse_usize(0..=10).parse("10"));
    println!("{:?}", parse_usize(0..=10).parse("11"));
}
