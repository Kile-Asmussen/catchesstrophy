use chumsky::{Parser, prelude::*};
use strum::{IntoEnumIterator, VariantNames};

use crate::model::*;

fn board_file<'s>() -> impl Parser<'s, &'s str, BoardFile> {
    one_of('a'..='h').map(|c| BoardFile::from_u8((c as u32 - 'a' as u32) as u8))
}

fn board_rank<'s>() -> impl Parser<'s, &'s str, BoardRank> {
    one_of('1'..='8').map(|c| BoardRank::from_u8((c as u32 - 'a' as u32) as u8))
}

fn square<'s>() -> impl Parser<'s, &'s str, Square> {
    board_file()
        .then(board_rank())
        .map(|(f, r)| Square::from_coords(f, r))
}

#[test]
fn test_square_parser() {
    for sq in Square::iter() {
        let sqs = sq.to_string();
        assert_eq!(
            square()
                .then_ignore(end())
                .parse(&sqs)
                .output()
                .expect(&format!("Unable to parse {}", sq)),
            &sq
        );
    }
}
