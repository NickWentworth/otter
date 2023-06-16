use crate::{board::Position, core::Color};

/// Evaluate the board position and assign a value representing a side's advantage
///
/// Positive implies white is winning, negative implies black is winning
pub fn evaluate(position: Position) -> f32 {
    let mut material = 0.0;

    let (white, black) = match position.active_color {
        Color::White => (position.active_pieces, position.inactive_pieces),
        Color::Black => (position.inactive_pieces, position.active_pieces),
    };

    for white_square in white {
        material += position.piece_list[white_square].unwrap().material_value();
    }
    for black_square in black {
        material -= position.piece_list[black_square].unwrap().material_value();
    }

    material
}
