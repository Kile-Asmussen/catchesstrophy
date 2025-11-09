//! # Extended FEN
//!
//! X-FEN is a FEN-inspired notation which is partially backwards-compatible with
//! plain FEN, while being able to also describe Chess960 positions, and
//! Capablanca Chess.
//!
//! Unfortunately since this crate does not support Capablanca Chess, this module
//! implements the subset of X-FEN notation that concerns itself with 64-square,
//! pawn-knight-bishop-rook-queen-king chess.
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

use chumsky::Parser;

use crate::{
    model::*,
    notation::{Parsable, fen::ColorCase},
};
use chumsky::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CastlingFile {
    Side(CastlingDirection),
    Explicit(BoardFile),
}

impl Parsable for ColorCase<CastlingFile> {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        choice((
            ColorCase::<BoardFile>::parser().map(|x| x.map(CastlingFile::Explicit)),
            ColorCase::<CastlingDirection>::parser().map(|x| x.map(CastlingFile::Side)),
        ))
    }
}
