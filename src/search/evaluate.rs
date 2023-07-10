use crate::board::Board;

use super::{pst::piece_square_table, Score};

/// Evaluate the board position and assign a value representing the active side's advantage
pub fn evaluate(board: &Board) -> Score {
    // initialize values used for evaluation
    let mut material = 0; // overall material advantage
    let mut position = 0; // positional advantage

    // for each side, add or subtract to values based on advantages
    for active_square in board.active_pieces() {
        let active_piece = board.piece_at(active_square).unwrap();

        material += active_piece.material_value();
        position += piece_square_table(active_piece, board.active_color(), active_square);
    }

    for inactive_square in board.inactive_pieces() {
        let inactive_piece = board.piece_at(inactive_square).unwrap();

        material -= inactive_piece.material_value();
        position -= piece_square_table(inactive_piece, board.inactive_color(), inactive_square);
    }

    material + position
}
