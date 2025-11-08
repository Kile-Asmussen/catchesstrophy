use chumsky::Parser;

use crate::{
    model::{BoardFile, ChessMan},
    notation::{Parsable, fen::ColorCase},
};
use chumsky::prelude::*;

impl Parsable for ColorCase<BoardFile> {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        use ColorCase::*;
        choice((
            BoardFile::parser().map(Black),
            one_of('A'..='H').map(|c| White(BoardFile::from_u8((c as u32 - 'H' as u32) as u8))),
        ))
        .labelled("expected on of A ... H, a ... h")
    }
}
