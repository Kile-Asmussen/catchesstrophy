#![allow(unused)]
#![feature(portable_simd)]
#![feature(duration_millis_float)]

use crate::model::{
    ChessMan, PseudoLegal, Square,
    attacking::FakeMoveEcharrayStrategy,
    bitboard::{BitBoard, ChessBoard, CompactBitBoard},
    hash::CompactZobristTables,
    mailbox::Mailbox,
    movegen::{BlessingStrategy, LegalBlessing, NoBlessing, enumerate},
    perft::{CloneMake, perft},
    vision::{MostlyBits, Panopticon},
};

#[test]
fn main_perft() {
    perft::<
        CompactBitBoard,
        MostlyBits,
        LegalBlessing<FakeMoveEcharrayStrategy, MostlyBits>,
        CloneMake,
        CompactZobristTables,
    >(5)
    .pretty_print();
}

/// Modeling the game of chess.
pub mod model;
