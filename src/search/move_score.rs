use crate::move_generator::Move;

use super::Score;

/// Contains a move and score related to that move
///
/// Used within evaluation algorithms to track the best move
pub struct MoveScore {
    mov: Move,
    score: Score,
}

impl From<Move> for MoveScore {
    fn from(value: Move) -> Self {
        MoveScore {
            mov: value,
            score: 0,
        }
    }
}
