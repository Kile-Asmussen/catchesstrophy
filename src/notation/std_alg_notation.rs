use std::io::empty;

use crate::{
    model::{BoardFile, BoardRank, ChessOfficer, PawnPromotion, Square},
    notation::{InCheck, Parsable, StdAlgCastling, StdAlgNotation, StdAlgOfficer, StdAlgPawn},
};
use chumsky::{container::Seq, prelude::*};

impl Parsable for StdAlgNotation {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        choice((
            StdAlgPawn::parser().map(Into::into),
            StdAlgOfficer::parser().map(Into::into),
            StdAlgCastling::parser().map(Into::into),
        ))
    }
}

impl Parsable for StdAlgPawn {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        group((
            BoardFile::parser().then_ignore(just('x')).or_not(),
            Square::parser(),
            just('=').ignore_then(pawn_promotion()).or_not(),
            InCheck::parser().or_not(),
        ))
        .map_group(Self::new)
    }
}

fn pawn_promotion<'s>() -> impl Parser<'s, &'s str, PawnPromotion> {
    use PawnPromotion::*;
    choice((
        just('N').to(KNIGHT),
        just('B').to(BISHOP),
        just('R').to(ROOK),
        just('Q').to(QUEEN),
    ))
}

impl Parsable for StdAlgOfficer {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        group((
            officer(),
            BoardFile::parser().or_not(),
            BoardRank::parser().or_not(),
            is_it(just('x')),
            Square::parser(),
            InCheck::parser().or_not(),
        ))
        .map_group(StdAlgOfficer::new)
    }
}

fn officer<'s>() -> impl Parser<'s, &'s str, ChessOfficer> {
    use ChessOfficer::*;
    choice((
        just('N').to(KNIGHT),
        just('B').to(BISHOP),
        just('R').to(ROOK),
        just('Q').to(QUEEN),
        just('K').to(KING),
    ))
}

pub fn is_it<'s, T>(p: impl Parser<'s, &'s str, T>) -> impl Parser<'s, &'s str, bool> {
    p.or_not().map(|s| s.is_some())
}

impl Parsable for StdAlgCastling {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        choice((
            just("O-O-O").ignore_then(InCheck::parser().or_not().map(StdAlgCastling::OOO)),
            just("O-O").ignore_then(InCheck::parser().or_not().map(StdAlgCastling::OO)),
        ))
    }
}

impl Parsable for InCheck {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        choice((just('+').to(InCheck::Check), just('#').to(InCheck::Mate)))
    }
}
