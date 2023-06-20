use crate::{board::Board, core::Color, search::Score};

/// Evaluate the board position and assign a value representing a side's advantage
pub fn evaluate(board: &Board) -> Score {
    let mut material = 0;

    let (white, black) = match board.active_color() {
        Color::White => (board.active_pieces(), board.inactive_pieces()),
        Color::Black => (board.inactive_pieces(), board.active_pieces()),
    };

    for white_square in white {
        material += board.piece_at(white_square).unwrap().material_value();
    }
    for black_square in black {
        material -= board.piece_at(black_square).unwrap().material_value();
    }

    material
}
