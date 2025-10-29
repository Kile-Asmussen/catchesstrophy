#![allow(unused)]
#![feature(portable_simd)]

use crate::model::{
    ChessMan, PseudoLegal, Square,
    bitboard::{ChessBoard, CompactBitBoard},
    hash::CompactZobristTables,
    mailbox::Mailbox,
    movegen::{NoBlessing, enumerate},
    vision::MostlyBits,
};

#[test]
fn main_perft() {
    // let mut board = [None; 64];
    // board[Square::c2.ix()] = Some(ChessMan::WHITE_PAWN);

    // let board = Mailbox(board);

    // let mut board = board.as_bitboard::<CompactBitBoard>();
    let board = CompactBitBoard::startpos::<CompactZobristTables>();

    let mut moves = vec![];

    enumerate::<CompactBitBoard, MostlyBits, NoBlessing>(&board, &mut moves);

    for i in moves {
        println!("{:?}", i);
    }
}

/// Modeling the game of chess.
pub mod model;
