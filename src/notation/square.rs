use chumsky::{Parser, prelude::*};
use strum::{IntoEnumIterator, VariantNames};

use crate::{
    model::*,
    notation::{Parsable, Prs},
};

impl Parsable for BoardFile {
    fn parser<'s>() -> impl Prs<'s, Self> {
        one_of('a'..='h')
            .map(|c| Self::from_u8((c as u32 - 'a' as u32) as u8))
            .labelled("a file letter a ... h")
            .boxed()
    }
}

impl Parsable for BoardRank {
    fn parser<'s>() -> impl Prs<'s, Self> {
        one_of('1'..='8')
            .map(|c| Self::from_u8((c as u32 - 'a' as u32) as u8))
            .labelled("a rank number 1 ... 8")
            .boxed()
    }
}

impl Parsable for Square {
    fn parser<'s>() -> impl Prs<'s, Self> {
        group((BoardFile::parser(), BoardRank::parser()))
            .map_group(Self::from_coords)
            .labelled("a valid chess board square a1 ... h8")
            .boxed()
    }
}

#[test]
fn test_square_parser() {
    for sq in Square::iter() {
        let sqs = sq.to_string();
        assert_eq!(
            Square::parser()
                .then_ignore(end())
                .parse(&sqs)
                .output()
                .expect(&format!("Unable to parse {}", sq)),
            &sq
        );
    }
}
