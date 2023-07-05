mod alpha_beta;
mod evaluate;
mod ordering;
mod tables;
mod tt;

pub use alpha_beta::{best_move, mate_in};

pub type TranspositionTable = tt::TranspositionTable<alpha_beta::ScoreData>;

/// Represents the score of the board, where a positive number implies moving side is ahead
pub type Score = i16;
