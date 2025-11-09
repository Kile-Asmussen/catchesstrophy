use chumsky::{Parser, prelude::*};

use crate::{
    model::*,
    notation::{CoordNotation, Parsable, Prs},
};

impl Parsable for CoordNotation {
    fn parser<'s>() -> impl Prs<'s, Self> {
        group((
            Square::parser(),
            Square::parser(),
            pawn_promotion().or_not(),
        ))
        .map_group(Self::new)
    }
}

fn pawn_promotion<'s>() -> impl Prs<'s, PawnPromotion> {
    use PawnPromotion::*;
    choice((
        just('n').to(KNIGHT),
        just('b').to(BISHOP),
        just('r').to(ROOK),
        just('q').to(QUEEN),
    ))
}
