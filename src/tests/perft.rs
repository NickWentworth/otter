use crate::{board::Board, move_generator::MoveGenerator};

/// Returns the number of positions possible from the given board state and given depth to search
///
/// Requires a pre-initialized move generator so that it can easily be re-used
pub fn perft(move_generator: &MoveGenerator, board: &mut Board, depth: u8) -> usize {
    if depth == 0 {
        1
    } else {
        let mut total = 0;

        for m in move_generator.generate_moves(board) {
            board.make_move(&m);
            total += perft(move_generator, board, depth - 1);
            board.unmake_move();
        }

        total
    }
}
