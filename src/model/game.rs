use std::collections::{HashMap, VecDeque};

use crate::model::{
    BitBoard, Legal, TransientInfo, ZOBHASHER,
    notation::{AlgNotaion, CoordNotation},
};

struct ChessMove {
    legal: Legal,
    coord: CoordNotation,
    alg: AlgNotaion,
    trans: TransientInfo,
    pre: u64,
    post: u64,
}

struct ChessGame {
    history: HashMap<u64, u8>,
    past: Vec<ChessMove>,
    future: VecDeque<ChessMove>,
    start: BitBoard,
    current: BitBoard,
}

fn pick_hash(hash: u64) -> u64 {
    hash.min(hash ^ ZOBHASHER.black_to_move)
}
