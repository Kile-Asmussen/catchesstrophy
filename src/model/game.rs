use std::{
    collections::{HashMap, VecDeque},
    hash::Hasher,
};

use crate::model::{
    BitMove, Legal, Transients,
    hash::ZobHasher,
    notation::{AlgNotaion, CoordNotation},
};
