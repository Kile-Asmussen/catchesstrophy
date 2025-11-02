//! # Modeling the game of chess.
//!
//! This module contains enums modeling values in chess,
//! as well as smore advanced representation details
//! in its sub-modules.

use strum::{EnumIs, FromRepr, VariantArray, VariantNames};

pub mod attacking;
pub mod binary;
pub mod board;
pub mod castling;
pub mod game;
pub mod hash;
pub mod movegen;
pub mod moving;
pub mod notation;
pub mod perft;
pub mod setup;
pub mod utils;
pub mod vision;
