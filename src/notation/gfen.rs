use crate::{
    model::{BoardFile, BoardRank, ChessColor, ChessMan, DataBoard, Square},
    notation::Parsable,
};

use chumsky::prelude::*;

/// Generalized Forsyth-Edwards Notation
///
/// The G-FEN is the notation that is the proper superset of
/// FEN, Shredder-FEN, and X-FEN.
///
/// This means:
///
/// - The board can be 8 ranks of 8 files with regular pieces
/// - Or it can be 8 ranks of 10 files with Princess and Empress pieces
/// - Castling can be specified by `KQkq` meaning implicitly the
///   outmost rook has the castling rights
/// - Or by directly specifying the file of the rook with the rights
/// - En-passant square is optional if en-passant is not possible
#[derive(Debug, Clone)]
pub struct GFen {
    pub board: FenBoard,
    pub active_player: ChessColor,
    pub castling: [[Option<CastlingFile>; 2]; 2],
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub turn: u16,
}

impl GFen {
    pub fn new(
        board: FenBoard,
        active_player: ChessColor,
        castling: [[Option<CastlingFile>; 2]; 2],
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

#[derive(Debug, Clone)]
pub enum FenBoard {
    Board64(DataBoard<Option<ChessMan>>),
    Board80([Option<ExtendedChessMan>; 80]),
}

impl Parsable for ChessMan {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
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
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(i8)]
pub enum ExtendedChessMan {
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

impl Parsable for ExtendedChessMan {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        use ExtendedChessMan::*;
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
    }
}

impl From<ChessMan> for ExtendedChessMan {
    fn from(value: ChessMan) -> Self {
        match value {
            ChessMan::BLACK_KING => Self::BLACK_KING,
            ChessMan::BLACK_QUEEN => Self::BLACK_QUEEN,
            ChessMan::BLACK_ROOK => Self::BLACK_ROOK,
            ChessMan::BLACK_BISHOP => Self::BLACK_BISHOP,
            ChessMan::BLACK_KNIGHT => Self::BLACK_KNIGHT,
            ChessMan::BLACK_PAWN => Self::BLACK_PAWN,
            ChessMan::WHITE_PAWN => Self::WHITE_PAWN,
            ChessMan::WHITE_KNIGHT => Self::WHITE_KNIGHT,
            ChessMan::WHITE_BISHOP => Self::WHITE_BISHOP,
            ChessMan::WHITE_ROOK => Self::WHITE_ROOK,
            ChessMan::WHITE_QUEEN => Self::WHITE_QUEEN,
            ChessMan::WHITE_KING => Self::WHITE_KING,
        }
    }
}

impl Parsable for [[Option<CastlingFile>; 2]; 2] {
    fn parser<'s>() -> impl chumsky::Parser<'s, &'s str, Self> {
        todo()
    }
}

pub enum ColorCase<T> {
    White(T),
    Black(T),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CastlingFile {
    Kingside,
    Queenside,
    ExplicitRank(BoardFile),
}
