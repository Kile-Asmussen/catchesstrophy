use std::{
    collections::{HashMap, VecDeque},
    hash::Hasher,
};

use crate::model::{
    BitBoard, BitMove, Legal, TransientInfo, ZOBRISTHASHES, ZobristHashes,
    notation::{AlgNotaion, CoordNotation},
};

pub struct ChessMove {
    legal: Legal,
    coord: CoordNotation,
    alg: AlgNotaion,
    trans: TransientInfo,
    pre: u64,
    post: u64,
}

pub struct ChessGame {
    history: HashMap<u64, u8, ZobristHashes>,
    past: Vec<ChessMove>,
    future: VecDeque<ChessMove>,
    start: BitBoard,
    current: BitBoard,
    moves: Vec<ChessMove>,
}

fn pick_hash(hash: u64) -> u64 {
    hash.min(hash ^ ZOBRISTHASHES.black_to_move)
}
