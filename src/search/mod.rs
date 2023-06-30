mod alpha_beta;
mod evaluate;
mod ordering;
mod tables;
mod tt;

pub use alpha_beta::alpha_beta;

pub type TranspositionTable = tt::TranspositionTable<alpha_beta::ScoreData>;

/// Represents the score of the board, where a positive number implies moving side is ahead
pub type Score = i32;

/// Score pertaining to a checkmate
///
/// Need +1 because `-Score::MIN` results in an overflow, as absolute value of Score::MAX < Score::MIN
pub const CHECKMATE: Score = Score::MIN + 1;

/// Score pertaining to a draw
pub const DRAW: Score = 0;
