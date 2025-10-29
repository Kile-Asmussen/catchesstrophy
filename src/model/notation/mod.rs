use std::fmt::{Display, write};

use strum::EnumIs;

use crate::model::{
    BitMove, CastlingDirection, ChessColor, ChessEchelon, ChessMan, ChessPawn, ChessPiece,
    ChessPromotion, EnPassant, SpecialMove, Square, Transients, VariantNames,
    castling::Castling,
    utils::{IteratorExtensions, SliceExtensions},
};

impl Square {
    #[inline]
    pub fn file(self) -> char {
        unsafe { char::from_u32_unchecked('a' as u32 + (self as u32 & 0x7)) }
    }

    #[inline]
    pub fn rank(self) -> u8 {
        1 + ((self as u8 & 0x38) >> 3)
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::VARIANTS[*self as usize])
    }
}

#[test]
fn square_file_rank() {
    for (i, sq1) in Square::VARIANTS.clones().enumerate() {
        let sq2 = Square::from_u8(i as u8);
        let sq2 = format!("{}{}", sq2.file(), sq2.rank());
        assert_eq!(sq1, &sq2);
    }
}

impl Display for ChessMan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arr = match (self.col(), f.alternate()) {
            (ChessColor::WHITE, true) => ["♙", "♘", "♗", "♖", "♕", "♔"],
            (ChessColor::WHITE, false) => ["♟", "♞", "♝", "♜", "♛", "♚"],
            (ChessColor::BLACK, true) => ["p", "n", "b", "r", "q", "k"],
            (ChessColor::BLACK, false) => ["P", "N", "B", "R", "Q", "K"],
        };
        write!(f, "{}", arr[self.ech().ix()])
    }
}

impl Display for ChessEchelon {
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
        <ChessEchelon as Display>::fmt(&ChessEchelon::from(*self), f)
    }
}

impl Display for ChessColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match *self {
                Self::WHITE => write!(f, "w"),
                Self::BLACK => write!(f, "b"),
            }
        } else {
            match *self {
                Self::WHITE => write!(f, "white"),
                Self::BLACK => write!(f, "black"),
            }
        }
    }
}

impl Display for ChessPromotion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#}", ChessEchelon::from(*self))
    }
}

impl Display for CastlingDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                Self::EAST => write!(f, "0-0-0"),
                Self::WEST => write!(f, "0-0"),
            }
        } else {
            match self {
                Self::EAST => write!(f, "O-O-O"),
                Self::WEST => write!(f, "O-O"),
            }
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
            let files = self.1.rook_start[ChessColor::WHITE.ix()].map(|s| s.file());
            [files.map(|c| c.to_ascii_uppercase()), files]
        } else {
            [['Q', 'K'], ['q', 'k']]
        };

        for c in [ChessColor::WHITE, ChessColor::BLACK] {
            for d in [CastlingDirection::WEST, CastlingDirection::EAST] {
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
    pub prom: Option<ChessPromotion>,
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
    Caslte(CastlingDirection, AlgCheck),
}

impl Display for AlgNotaion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pawn(p, x) => write!(f, "{}{}", p, x),
            Self::Piece(p, x) => write!(f, "{}{}", p, x),
            Self::Caslte(p, x) => write!(f, "{}{}", p, x),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlgPawn {
    pub from: Square,
    pub to: Square,
    pub capture: bool,
    pub promote: Option<ChessPromotion>,
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
    pub piece: ChessMan,
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
                ChessEchelon::from(self.piece),
                self.from,
                if self.capture { "x" } else { "" },
                self.to
            )?;
        } else {
            write!(f, "{}", ChessEchelon::from(self.piece))?;
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
            && (self.prom.is_none() || self.prom == ChessPromotion::from_special(mv.special))
    }
}

impl MoveMatcher for AlgNotaion {
    fn matches(self, mv: BitMove) -> bool {
        match self {
            Self::Pawn(p, _) => {
                mv.ech == ChessEchelon::PAWN
                    && (p.from as u8 & 0x7) == (mv.from as u8 & 0x7)
                    && p.to == mv.to
                    && p.capture == mv.capture.is_some()
                    && p.promote == ChessPromotion::from_special(mv.special)
            }
            Self::Piece(p, _) => {
                mv.ech == ChessEchelon::from(p.piece)
                    && mv.to == p.to
                    && p.capture == mv.capture.is_some()
                    && match p.disambiguate {
                        (false, false) => true,
                        (true, false) => (p.from as u8 & 0x7) == (mv.from as u8 & 0x7),
                        (false, true) => (p.from as u8 & 0x38) == (mv.from as u8 & 0x38),
                        (true, true) => p.from == mv.from,
                    }
            }
            Self::Caslte(c, _) => mv.special == Some(SpecialMove::from(c)),
        }
    }
}
