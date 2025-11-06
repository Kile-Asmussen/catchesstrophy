pub mod fen;

use std::fmt::Display;

use strum::VariantNames;

use crate::model::{CastlingDirection, ChessMove, PawnPromotion, Square};

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Square::VARIANTS[self.ix()])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoordNotation {
    from: Square,
    to: Square,
    prom: Option<PawnPromotion>,
}

impl From<ChessMove> for CoordNotation {
    fn from(value: ChessMove) -> Self {
        Self {
            from: value.from,
            to: value.to,
            prom: PawnPromotion::from_special(value.special),
        }
    }
}

impl Display for CoordNotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.from,
            self.to,
            ["", "", "n", "b", "r", "q"][self.prom.map(|x| x.ix()).unwrap_or(0)]
        )
    }
}

pub enum AlgNotation {
    PawnMove(),
    OfficerMove(),
    CastlingMove(CastlingDirection, InCheck),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InCheck {
    Check,
    CheckMate,
}
