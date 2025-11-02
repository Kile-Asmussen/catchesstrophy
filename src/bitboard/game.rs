use std::{
    collections::{HashMap, VecDeque},
    hash::Hasher,
};

use crate::bitboard::{
    BitMove, LegalMove, Transients,
    hash::ZobHasher,
    notation::{AlgNotaion, CoordNotation},
};
