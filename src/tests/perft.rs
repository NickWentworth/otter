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

/// Returns identical value to perft function, but prints the perft of every move from starting position
pub fn perft_divide(move_generator: &MoveGenerator, board: &mut Board, depth: u8) -> usize {
    if depth == 0 {
        1
    } else {
        let mut total = 0;

        for m in move_generator.generate_moves(board) {
            board.make_move(&m);

            let this_move_total = perft(move_generator, board, depth - 1);
            total += this_move_total;
            println!("{}: {}", m, this_move_total);

            board.unmake_move();
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{board::Board, move_generator::MoveGenerator};

    /// Varying positions with expected values from the perft function at a given depth
    ///
    /// Fetched from https://www.chessprogramming.org/Perft_Results, depth is chosen to have expected value ~10 million nodes
    ///
    /// Tuple is in the form (fen, depth, expected)
    const TEST_CASES: [(&str, u8, usize); 3] = [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            5,
            4865609,
        ),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            4,
            4085603,
        ),
        (
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            5,
            15833292,
        ),
    ];

    #[test]
    fn test_perft() {
        let mg = MoveGenerator::new();

        for (fen, depth, expected) in TEST_CASES {
            let mut b = Board::new(fen.to_string());

            assert_eq!(
                perft(&mg, &mut b, depth),
                expected,
                "Failed on position {} at depth {}",
                fen,
                depth
            );
        }
    }
}
