mod alpha_beta;
mod evaluate;
mod ordering;
mod pst;
mod tt;

pub use alpha_beta::{Searcher, SearchTT};

/// Represents the score of the board, where a positive number implies moving side is ahead
pub type Score = i16;
