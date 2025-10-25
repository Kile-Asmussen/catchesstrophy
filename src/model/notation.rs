use std::fmt::Display;

use strum::EnumIs;

use crate::model::{
    Castles, Castling, Color, Piece, Promotion, Rights, Square, TransientInfo, VariantNames,
};

impl Square {
    fn file(self) -> char {
        unsafe { char::from_u32_unchecked('a' as u32 + (self as u32 & 0x7)) }
    }

    fn rank(self) -> u8 {
        (self as u8 & 0x38) >> 3
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::VARIANTS[*self as usize])
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arr = if f.alternate() {
            ["", "p", "n", "b", "r", "q", "k"]
        } else {
            ["", "P", "N", "B", "R", "Q", "K"]
        };
        write!(f, "{}", arr[*self as usize])
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
        write!(f, "{:#}", Piece::from(*self))
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

impl Display for Rights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, c) in [(0, 'K'), (1, 'Q'), (2, 'k'), (3, 'q')] {
            if (self.0 & 1 << i) != 0 {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rights960(pub Rights, pub Square, pub Square);

impl Rights960 {
    fn from(r: Rights, c: &Castling) -> Self {
        Self(r, c.rook_from[0], c.rook_from[1])
    }
}

impl Display for Rights960 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chars = [
            (1, self.2.file().to_ascii_uppercase()),
            (0, self.1.file().to_ascii_uppercase()),
            (3, self.1.file()),
            (2, self.2.file()),
        ];
        for (i, c) in chars {
            if (self.0.0 & 1 << i) != 0 {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

impl Display for TransientInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.rights,
            self.ep_square
                .map(|s| Square::VARIANTS[s as usize])
                .unwrap_or("-"),
            self.halfmove_clock,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TransientInfo960 {
    pub rights: Rights960,
    pub ep_square: Option<Square>,
    pub halfmove_clock: u8,
}

impl TransientInfo960 {
    fn from(t: TransientInfo, c: &Castling) -> Self {
        Self {
            rights: Rights960::from(t.rights, c),
            ep_square: t.ep_square,
            halfmove_clock: t.halfmove_clock,
        }
    }
}

impl Display for TransientInfo960 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.rights,
            self.ep_square
                .map(|s| Square::VARIANTS[s as usize])
                .unwrap_or("-"),
            self.halfmove_clock,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoordNotation {
    pub from: Square,
    pub to: Square,
    pub prom: Promotion,
}

impl Display for CoordNotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.from, self.to, self.prom)
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
    pub promote: Promotion,
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

        if self.promote != Promotion::NONE {
            write!(f, "={}", self.promote)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlgPiece {
    pub piece: Piece,
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
                self.piece,
                self.from,
                if self.capture { "x" } else { "" },
                self.to
            )?;
        } else {
            write!(f, "{}", self.piece)?;
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
        .map(|x| format!("{:08b}", x.reverse_bits()))
        .join("\n")
        .replace('1', "██")
        .replace('0', "  ")
}
