#![allow(unused)]
#![feature(portable_simd)]
#![feature(duration_millis_float)]

use crate::bitboard::{
    attacking::FakeMoveSimplStrategy,
    board::{BitBoard, ChessBoard, CompactBitBoard, FullBitBoard, FullerBitBoard},
    hash::{CompactZobristTables, FullZobristTables},
    movegen::{BlessingStrategy, LegalBlessing, NoBlessing, enumerate},
    perft::{CloneMake, HashMapMemo, MakeUnmake, perft},
    setup::SimpleBoard,
    vision::{MostlyBits, Panopticon},
};

#[test]
fn main_perft() {
    println!("Fuller:");
    perft::<
        FullerBitBoard,
        MostlyBits,
        LegalBlessing<FakeMoveSimplStrategy<MostlyBits>>,
        CloneMake,
        FullZobristTables,
    >(5, false, ())
    .pretty_print();

    println!("\nFull:");
    perft::<
        FullBitBoard,
        MostlyBits,
        LegalBlessing<FakeMoveSimplStrategy<MostlyBits>>,
        CloneMake,
        FullZobristTables,
    >(5, false, ())
    .pretty_print();

    println!("\nCompact:");
    perft::<
        CompactBitBoard,
        MostlyBits,
        LegalBlessing<FakeMoveSimplStrategy<MostlyBits>>,
        CloneMake,
        FullZobristTables,
    >(5, false, ())
    .pretty_print();
}

/// Modeling the game of chess.
pub mod bitboard;
pub mod model;
pub mod notation;
