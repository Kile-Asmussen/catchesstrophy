//! # Extended FEN
//!
//! X-FEN is a FEN-inspired notation which is partially backwards-compatible with
//! plain FEN, while being able to also describe Chess960 positions, and
//! 10x8 Knighted Chess.
//!
//! Unfortunately since this crate does not at all support Knighted Chess, except
//! parsing it, in this module.
//!
//! X-FEN handles alternate castling by implementing Shredder-FEN's convention of
//! indicating which rook has castling rights by the file letter, but only in the
//! case where there is more than one rook on the same side of the king, and it
//! is not the rook most distant to the king which has the castling right.
//!
//! X-FEN is incompatible with ordinary FEN by restricting the presence of the
//! en-passant square to only be specified when en-passant is actually possible.
//!
//! X-FEN is, from a protocol design point of view, badly designed. It is not
//! backwards compatible with FEN, and is implementationally complicated, as it
//! requires nontrivial sanity checks.

use std::fmt::Display;

use chumsky::Parser;
use strum::VariantArray;

use crate::{
    model::*,
    notation::{
        Parsable, Prs,
        fen::{
            ColorCase, fen_board, fen_color, fen_epc_square, fen_halfmove, fen_turn,
            generalized::{gfen_board, gfen_castling, gfen_epc_square, parse_usize},
            ws,
        },
    },
};
use chumsky::prelude::*;

pub struct StdExtFenBoard {
    pub board: DataBoard<Option<ChessMan>>,
    pub to_move: ChessColor,
    pub castling_rights: Vec<ColorCase<CastlingFile>>,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl StdExtFenBoard {
    pub fn new(
        board: DataBoard<Option<ChessMan>>,
        to_move: ChessColor,
        castling_rights: Vec<ColorCase<CastlingFile>>,
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

impl Parsable for StdExtFenBoard {
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

pub struct KnightedExtFenBoard {
    pub board: KnightedDataBoard,
    pub to_move: ChessColor,
    pub castling_rights: Vec<ColorCase<KnightedCastlingFile>>,
    pub en_passant: Option<(KnightedBoardFile, u8)>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl KnightedExtFenBoard {
    pub fn new(
        board: KnightedDataBoard,
        to_move: ChessColor,
        castling_rights: Vec<ColorCase<KnightedCastlingFile>>,
        en_passant: Option<(KnightedBoardFile, u8)>,
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

impl Parsable for KnightedExtFenBoard {
    fn parser<'s>() -> impl Prs<'s, Self> {
        group((
            xfen_board().then_ignore(ws()),
            fen_color().then_ignore(ws()),
            gfen_castling().then_ignore(ws()),
            xfen_epc_square().then_ignore(ws()),
            fen_halfmove().then_ignore(ws()),
            fen_turn(),
        ))
        .padded()
        .map_group(Self::new)
        .boxed()
    }
}

fn xfen_board<'s>() -> impl Prs<'s, KnightedDataBoard> {
    gfen_board(10..=10, 8..=8, xfen_knighted_chessman())
        .map(|v| {
            let mut b = [None; 80];
            b.clone_from_slice(&v.concat());
            KnightedDataBoard(b)
        })
        .boxed()
}

fn xfen_epc_square<'s>() -> impl Prs<'s, Option<(KnightedBoardFile, u8)>> {
    choice((
        KnightedBoardFile::parser()
            .then(parse_usize(1..=8).map(|u| u as u8))
            .map(Some),
        just('-').to(None),
    ))
    .boxed()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CastlingFile {
    Side(CastlingDirection),
    Explicit(BoardFile),
}

/// Extension of the chessboard to 80 squares, 8 ranks and 10 files.
#[derive(Debug, Clone)]
pub struct KnightedDataBoard(pub [Option<KnightedChessMan>; 80]);

/// Extension of ordinary chessmen to also include the
/// princess (knight + bishop, aka. archbishop) and empress
/// (rook + knight, aka. chansellor) as used in Knighted Chess
/// variants.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantArray, Hash)]
#[repr(i8)]
pub enum KnightedChessMan {
    BLACK_KING = -8,
    BLACK_QUEEN = -7,
    BLACK_EMPRESS = -6,
    BLACK_PRINCESS = -5,
    BLACK_ROOK = -4,
    BLACK_BISHOP = -3,
    BLACK_KNIGHT = -2,
    BLACK_PAWN = -1,
    WHITE_PAWN = 1,
    WHITE_KNIGHT = 2,
    WHITE_BISHOP = 3,
    WHITE_ROOK = 4,
    WHITE_PRINCESS = 5,
    WHITE_EMPRESS = 6,
    WHITE_QUEEN = 7,
    WHITE_KING = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KnightedCastlingFile {
    Side(CastlingDirection),
    Explicit(KnightedBoardFile),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum KnightedBoardFile {
    a_ = 0,
    b_ = 1,
    c_ = 2,
    d_ = 3,
    e_ = 4,
    f_ = 5,
    g_ = 6,
    h_ = 7,
    i_ = 8,
    j_ = 9,
}

impl KnightedBoardFile {
    pub const VARIANTS: &'static [&'static str] =
        &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

    /// Use this file as an array index.
    #[inline]
    pub fn ix(self) -> usize {
        self as usize
    }

    /// Infallible conversion from a u8 by way of truncating the
    /// extraneous bits.
    #[inline]
    pub fn from_u8(ix: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(ix % 10) }
    }
}

impl Display for KnightedBoardFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::VARIANTS[self.ix()])
    }
}

impl Parsable for KnightedBoardFile {
    fn parser<'s>() -> impl Prs<'s, Self> {
        one_of('a'..='j')
            .map(|c| Self::from_u8((c as u32 - 'a' as u32) as u8))
            .labelled("a file letter a ... j")
    }
}

impl Parsable for ColorCase<KnightedBoardFile> {
    fn parser<'s>() -> impl Prs<'s, Self> {
        use ColorCase::*;
        choice((
            KnightedBoardFile::parser().map(Black),
            one_of('A'..='J')
                .map(|c| White(KnightedBoardFile::from_u8((c as u32 - 'H' as u32) as u8))),
        ))
        .labelled("one of A ... J, a ... j")
        .boxed()
    }
}

impl Parsable for ColorCase<CastlingFile> {
    fn parser<'s>() -> impl Prs<'s, Self> {
        choice((
            ColorCase::<BoardFile>::parser().map(|x| x.map(CastlingFile::Explicit)),
            ColorCase::<CastlingDirection>::parser().map(|x| x.map(CastlingFile::Side)),
        ))
        .boxed()
    }
}

impl Parsable for ColorCase<KnightedCastlingFile> {
    fn parser<'s>() -> impl Prs<'s, Self> {
        choice((
            ColorCase::<KnightedBoardFile>::parser().map(|x| x.map(KnightedCastlingFile::Explicit)),
            ColorCase::<CastlingDirection>::parser().map(|x| x.map(KnightedCastlingFile::Side)),
        ))
        .boxed()
    }
}

fn xfen_knighted_chessman<'s>() -> impl Prs<'s, KnightedChessMan> {
    use KnightedChessMan::*;
    choice((
        just('k').to(BLACK_KING),
        just('q').to(BLACK_QUEEN),
        just('c').to(BLACK_EMPRESS),
        just('a').to(BLACK_PRINCESS),
        just('r').to(BLACK_ROOK),
        just('b').to(BLACK_BISHOP),
        just('n').to(BLACK_KNIGHT),
        just('p').to(BLACK_PAWN),
        just('P').to(WHITE_PAWN),
        just('N').to(WHITE_KNIGHT),
        just('B').to(WHITE_BISHOP),
        just('R').to(WHITE_ROOK),
        just('A').to(WHITE_PRINCESS),
        just('C').to(WHITE_EMPRESS),
        just('Q').to(WHITE_QUEEN),
        just('K').to(WHITE_KING),
    ))
    .labelled("expected one of PNBRACQKpnbracqk")
    .boxed()
}
