use std::fmt::{Display, write};

use strum::EnumIs;

use crate::model::{
    BitMove, CLASSIC_CASTLING, Castles, Castling, ChessMan, ChessPawn, ChessPiece, Color,
    EnPassant, Promotion, Special, Square, Transients, VariantNames,
};

impl Square {
    #[inline]
    pub fn file(self) -> char {
        unsafe { char::from_u32_unchecked('a' as u32 + (self as u32 & 0x7)) }
    }

    #[inline]
    pub fn rank(self) -> u8 {
        (self as u8 & 0x38) >> 3
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::VARIANTS[*self as usize])
    }
}

impl Display for ChessMan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arr = if f.alternate() {
            ["p", "n", "b", "r", "q", "k"]
        } else {
            ["P", "N", "B", "R", "Q", "K"]
        };
        write!(f, "{}", arr[self.ix()])
    }
}

impl Display for ChessPawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <ChessMan as Display>::fmt(&ChessMan::from(*self), f)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::WHITE => write!(f, "w"),
            Self::BLACK => write!(f, "b"),
        }
    }
}

impl Display for Promotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#}", ChessMan::from(*self))
    }
}

impl Display for Castles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EAST => write!(f, "O-O"),
            Self::WEST => write!(f, "O-O-O"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Rights([[bool; 2]; 2], &'static Castling);

impl Display for Rights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == [[false; 2]; 2] {
            return write!(f, "-");
        }

        let letters = if self.1.chess960 {
            let files = self.1.rook_from.map(|s| s.file());
            [files.map(|c| c.to_ascii_uppercase()), files]
        } else {
            [['Q', 'K'], ['q', 'k']]
        };

        for c in [Color::WHITE, Color::BLACK] {
            for d in [Castles::WEST, Castles::EAST] {
                write!(f, "{}", letters[c.ix()][d.ix()])?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct TransientInfo {
    pub rights: Rights,
    pub en_passant: Option<EnPassant>,
    pub halfmove_clock: u8,
}

impl TransientInfo {
    fn from(t: Transients, c: &'static Castling) -> Self {
        Self {
            rights: Rights(t.rights, c),
            en_passant: t.en_passant,
            halfmove_clock: t.halfmove_clock,
        }
    }
}

impl Display for TransientInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.rights,
            self.en_passant
                .map(|e| Square::VARIANTS[e.square.ix()])
                .unwrap_or("-"),
            self.halfmove_clock,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordNotation {
    pub from: Square,
    pub to: Square,
    pub prom: Option<Promotion>,
}

impl Display for CoordNotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(prom) = self.prom {
            write!(f, "{}", prom)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIs)]
pub enum AlgNotaion {
    Pawn(AlgPawn, AlgCheck),
    Piece(AlgPiece, AlgCheck),
    Caslte(Castles, AlgCheck),
}

impl Display for AlgNotaion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pawn(p, c) => write!(f, "{}{}", p, c),
            Self::Piece(p, c) => write!(f, "{}{}", p, c),
            Self::Caslte(p, c) => write!(f, "{}{}", p, c),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlgPawn {
    pub from: Square,
    pub to: Square,
    pub capture: bool,
    pub promote: Option<Promotion>,
}

impl Display for AlgPawn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "P{}{}{}",
                self.from,
                if self.capture { "x" } else { "" },
                self.to
            )?;
        } else {
            if self.capture {
                write!(f, "{}x", self.from.file())?;
            }
            write!(f, "{}", self.to)?;
        }

        if let Some(promote) = self.promote {
            write!(f, "={}", promote)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlgPiece {
    pub piece: ChessPiece,
    pub from: Square,
    pub to: Square,
    pub capture: bool,
    pub disambiguate: (bool, bool),
}

impl Display for AlgPiece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{}{}{}{}",
                ChessMan::from(self.piece),
                self.from,
                if self.capture { "x" } else { "" },
                self.to
            )?;
        } else {
            write!(f, "{}", ChessMan::from(self.piece))?;
            if self.disambiguate.0 {
                write!(f, "{}", self.from.file())?;
            }
            if self.disambiguate.1 {
                write!(f, "{}", self.from.rank())?;
            }
            if self.capture {
                write!(f, "x")?;
            }
            write!(f, "{}", self.to)?;
        }

        Ok(())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(u8)]
pub enum AlgCheck {
    #[default]
    NONE = 0,
    CHECK = 1,
    MATE = 2,
}

impl Display for AlgCheck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NONE => {}
            Self::CHECK => write!(f, "+")?,
            Self::MATE => write!(f, "#")?,
        }
        Ok(())
    }
}

pub fn show_mask(mask: u64) -> String {
    mask.to_be_bytes()
        .iter()
        .enumerate()
        .map(|(i, x)| {
            format!("{:08b}", x.reverse_bits())
                .replace('1', "██")
                .replace('0', "  ")
                + &(8 - i).to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\na b c d e f g h "
}

trait MoveMatcher {
    fn matches(self, mv: BitMove) -> bool;
}

impl MoveMatcher for CoordNotation {
    fn matches(self, mv: BitMove) -> bool {
        self.from == mv.from
            && self.to == mv.to
            && (self.prom.is_none() || self.prom == Promotion::from_special(mv.special))
    }
}

impl MoveMatcher for AlgNotaion {
    fn matches(self, mv: BitMove) -> bool {
        match self {
            Self::Pawn(p, _) => {
                mv.man == ChessMan::PAWN
                    && (p.from as u8 & 0x7) == (mv.from as u8 & 0x7)
                    && p.to == mv.to
                    && p.capture == mv.capture.is_some()
                    && p.promote == Promotion::from_special(mv.special)
            }
            Self::Piece(p, _) => {
                mv.man == ChessMan::from(p.piece)
                    && mv.to == p.to
                    && p.capture == mv.capture.is_some()
                    && match p.disambiguate {
                        (false, false) => true,
                        (true, false) => (p.from as u8 & 0x7) == (mv.from as u8 & 0x7),
                        (false, true) => (p.from as u8 & 0x38) == (mv.from as u8 & 0x38),
                        (true, true) => p.from == mv.from,
                    }
            }
            Self::Caslte(c, _) => mv.special == Some(Special::from(c)),
        }
    }
}
