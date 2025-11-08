use chumsky::{Parser, prelude::*};
use strum::{IntoEnumIterator, VariantNames};

use crate::{model::*, notation::Parsable};

impl Parsable for BoardFile {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        one_of('a'..='h').map(|c| Self::from_u8((c as u32 - 'a' as u32) as u8))
    }
}

impl Parsable for BoardRank {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        one_of('1'..='8').map(|c| Self::from_u8((c as u32 - 'a' as u32) as u8))
    }
}

impl Parsable for Square {
    fn parser<'s>() -> impl Parser<'s, &'s str, Self> {
        group((BoardFile::parser(), BoardRank::parser())).map_group(Self::from_coords)
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
