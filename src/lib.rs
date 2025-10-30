#![allow(unused)]
#![feature(portable_simd)]
#![feature(duration_millis_float)]

use crate::model::{
    ChessMan, PseudoLegal, Square,
    attacking::FakeMoveEcharrayStrategy,
    bitboard::{BitBoard, ChessBoard, CompactBitBoard, FullBitBoard, FullerBitBoard},
    hash::{CompactZobristTables, FullZobristTables},
    mailbox::Mailbox,
    movegen::{BlessingStrategy, LegalBlessing, NoBlessing, enumerate},
    perft::{CloneMake, HashMapMemo, MakeUnmake, perft},
    vision::{MostlyBits, Panopticon},
};

#[test]
fn main_perft() {
    println!("Fuller");
    perft::<
        FullerBitBoard,
        MostlyBits,
        LegalBlessing<FakeMoveEcharrayStrategy, MostlyBits>,
        CloneMake,
        FullZobristTables,
    >(1, false, ())
    .pretty_print();
}

/// Modeling the game of chess.
pub mod model;
