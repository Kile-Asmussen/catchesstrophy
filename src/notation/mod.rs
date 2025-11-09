pub mod coordinate;
pub mod fen;
pub mod square;
pub mod stdalg;

use std::{
    fmt::{Display, Write},
    os::unix::process,
};

use chumsky::{Parser, error::Rich, extra::Err};
use strum::VariantNames;
use trie_rs::inc_search;

use crate::model::{
    BoardFile, BoardRank, CastlingDirection, ChessMove, ChessOfficer, PawnPromotion, Square,
};

pub trait Prs<'s, O> = Parser<'s, &'s str, O, Err<Rich<'s, char>>>;

pub trait Parsable: Sized {
    fn parser<'s>() -> impl Prs<'s, Self>;
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::VARIANTS[self.ix()])
    }
}

impl Display for BoardFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::VARIANTS[self.ix()])
    }
}

impl Display for BoardRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::VARIANTS[self.ix()])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoordNotation {
    pub from: Square,
    pub to: Square,
    pub prom: Option<PawnPromotion>,
}

impl CoordNotation {
    pub fn new(from: Square, to: Square, prom: Option<PawnPromotion>) -> Self {
        Self { from, to, prom }
    }
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
        self.from.fmt(f)?;
        self.to.fmt(f)?;
        f.write_str(["", "n", "b", "r", "q"][self.prom.map(|x| x.ix()).unwrap_or(0)])?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StdAlgNotation {
    Pawn(StdAlgPawn),
    Officer(StdAlgOfficer),
    Castling(StdAlgCastling),
}

impl From<StdAlgCastling> for StdAlgNotation {
    fn from(value: StdAlgCastling) -> Self {
        Self::Castling(value)
    }
}

impl From<StdAlgPawn> for StdAlgNotation {
    fn from(value: StdAlgPawn) -> Self {
        Self::Pawn(value)
    }
}

impl From<StdAlgOfficer> for StdAlgNotation {
    fn from(value: StdAlgOfficer) -> Self {
        Self::Officer(value)
    }
}

impl StdAlgNotation {
    pub const OFFICERS: &'static [&'static str] = &["", "N", "B", "R", "Q", "K"];
}

impl Display for StdAlgNotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pawn(alg_pawn_move) => alg_pawn_move.fmt(f),
            Self::Officer(alg_officer_move) => alg_officer_move.fmt(f),
            Self::Castling(alg_castling_move) => alg_castling_move.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StdAlgPawn {
    to: Square,
    capture: Option<BoardFile>,
    promotion: Option<PawnPromotion>,
    in_check: Option<InCheck>,
}

impl StdAlgPawn {
    pub fn new(
        capture: Option<BoardFile>,
        to: Square,
        promotion: Option<PawnPromotion>,
        in_check: Option<InCheck>,
    ) -> Self {
        Self {
            to,
            capture,
            promotion,
            in_check,
        }
    }
}

impl Display for StdAlgPawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = self.capture {
            c.fmt(f)?;
            f.write_char('x')?;
        }

        self.to.fmt(f)?;

        if let Some(p) = self.promotion {
            f.write_char('=')?;
            f.write_str(StdAlgNotation::OFFICERS[p.ix()])?;
        }

        if let Some(in_check) = self.in_check {
            in_check.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StdAlgOfficer {
    officer: ChessOfficer,
    from_file: Option<BoardFile>,
    from_rank: Option<BoardRank>,
    capture: bool,
    to: Square,
    in_check: Option<InCheck>,
}

impl StdAlgOfficer {
    pub fn new(
        officer: ChessOfficer,
        from_file: Option<BoardFile>,
        from_rank: Option<BoardRank>,
        capture: bool,
        to: Square,
        in_check: Option<InCheck>,
    ) -> Self {
        Self {
            officer,
            from_file,
            from_rank,
            capture,
            to,
            in_check,
        }
    }
}

impl Display for StdAlgOfficer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(StdAlgNotation::OFFICERS[self.officer.ix()])?;

        if let Some(d) = self.from_file {
            d.fmt(f)?;
        }

        if let Some(d) = self.from_rank {
            d.fmt(f)?;
        }

        if self.capture {
            f.write_char('x');
        }

        self.to.fmt(f)?;

        if let Some(in_check) = self.in_check {
            in_check.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum StdAlgCastling {
    OOO(Option<InCheck>),
    OO(Option<InCheck>),
}

impl Display for StdAlgCastling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::OOO(c) => {
                f.write_str("O-O-O")?;
                *c
            }
            Self::OO(c) => {
                f.write_str("O-O")?;
                *c
            }
        };

        if let Some(c) = c {
            c.fmt(f)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum InCheck {
    Check,
    Mate,
}

impl Display for InCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Check => f.write_str("+"),
            Self::Mate => f.write_str("#"),
        }
    }
}
