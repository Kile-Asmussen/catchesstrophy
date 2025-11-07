use chumsky::{Parser, prelude::one_of};
use strum::VariantNames;

use crate::model::Square;

fn board_file<'s>() -> impl Parser<'s, &'s str, BoardRank> {
    one_of('a'..='h').map(|c| (c as u32 - 'a' as u32))
}

fn board_rank<'s>() -> impl Parser<'s, &'s str, BoardRank> {

}

fn square<'s>() -> impl Parser<'s, &'s str, Square> {
    one_of('a'..='h').then(one_of('1'..='8')).map(|(f, r)| {
        let f = f as i32;
        let r = r as i32;
        Square::from_u8((f - ('a' as i32) + (r - ('1' as i32)) << 3) as u8)
    })
}

fn promotion<'s>() -> impl Parser<'s, &'s str, Square> {
    one_of("nbrq").map(|c| )
}
