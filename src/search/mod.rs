mod evaluate;

// searching algorithms
mod minimax;
// TODO - alpha-beta

pub use minimax::minimax;

/// Represents the score of the board, + implies white is ahead, - implies black is ahead
pub type Score = i32;

pub const CHECKMATE_WHITE: Score = Score::MAX;
pub const CHECKMATE_BLACK: Score = Score::MIN;

pub const DRAW: Score = 0;
