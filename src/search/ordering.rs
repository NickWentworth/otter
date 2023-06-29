use crate::{
    board::{Move, MoveFlag},
    core::Piece,
};

// TODO - as of now, sort_by_cached_key is slower than sort_by_key, if this importance calculation grows, it may change

/// Orders the moves in a given list according to the likelihood of the move being good
pub fn order_moves(moves: &mut Vec<Move>) {
    // generate an approximate importance value per move and sort by it
    moves.sort_by_key(|mov| {
        use MoveFlag::*;

        let mut importance = 0;

        let moving_value = mov.piece.material_value();

        let attacked_value = match mov.flag {
            Capture(piece) => piece.material_value(),
            CapturePromotion(piece, _) => piece.material_value(),
            EnPassantCapture(_) => Piece::Pawn.material_value(),
            _ => 0,
        };

        // prefer attacking valuable opposing pieces with less valuable friendly pieces
        if attacked_value != 0 {
            importance += (5 * attacked_value) - moving_value;
        }

        // prefer promotions
        importance += match mov.flag {
            Promotion(promoted_piece) => promoted_piece.material_value(),
            CapturePromotion(_, promoted_piece) => promoted_piece.material_value(),
            _ => 0,
        };

        importance
    });

    // finally, reverse the ordering of moves because we want highest importance first
    moves.reverse();
}
