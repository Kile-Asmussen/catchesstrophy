use std::{
    collections::{HashMap, VecDeque},
    hash::Hasher,
};

use crate::bitboard::{
    ChessMove, LegalMove, Transients,
    hash::ZobHasher,
    notation::{AlgNotaion, CoordNotation},
};
