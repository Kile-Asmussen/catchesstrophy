//! # Shredder-FEN
//!
//! S-FEN is a FEN variant which is incompatible with standard FEN,
//! but accounts for Chess960 and variants, by directly specifying the
//! file of the rook that has castling rights. Otherwise it is identical
//! to FEN.
//!
//! The starting position of a standard game of chess in S-FEN is thus:
//! ```text
//! rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w AHah - 0 1
//! ```

use chumsky::Parser;

use crate::{
    model::{BoardFile, ChessColor, ChessMan, DataBoard, Square},
    notation::{
        Parsable, Prs,
        fen::{
            ColorCase, fen_board, fen_color, fen_epc_square, fen_halfmove, fen_turn,
            generalized::gfen_castling, ws,
        },
    },
};
use chumsky::prelude::*;

#[derive(Debug, Clone)]
pub struct ShrFenBoard {
    pub board: DataBoard<Option<ChessMan>>,
    pub to_move: ChessColor,
    pub castling_rights: Vec<ColorCase<BoardFile>>,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl ShrFenBoard {
    pub fn new(
        board: DataBoard<Option<ChessMan>>,
        to_move: ChessColor,
        castling_rights: Vec<ColorCase<BoardFile>>,
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

impl Parsable for ShrFenBoard {
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

impl Parsable for ColorCase<BoardFile> {
    fn parser<'s>() -> impl Prs<'s, Self> {
        use ColorCase::*;
        choice((
            BoardFile::parser().map(Black),
            one_of('A'..='H').map(|c| White(BoardFile::from_u8((c as u32 - 'H' as u32) as u8))),
        ))
        .labelled("on of A ... H, a ... h")
    }
}
