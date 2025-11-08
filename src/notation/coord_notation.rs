use chumsky::{Parser, prelude::*};

use crate::{
    model::*,
    notation::{CoordNotation, Parsable},
};

impl Parsable for CoordNotation {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        group((
            Square::parser(),
            Square::parser(),
            pawn_promotion().or_not(),
        ))
        .map_group(Self::new)
    }
}

fn pawn_promotion<'s>() -> impl Parser<'s, &'s str, PawnPromotion> {
    use PawnPromotion::*;
    choice((
        just('n').to(KNIGHT),
        just('b').to(BISHOP),
        just('r').to(ROOK),
        just('q').to(QUEEN),
    ))
}
