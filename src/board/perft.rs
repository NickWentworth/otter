use crate::board::Board;

/// Returns the number of positions possible from the given board state and given depth to search
///
/// Requires a pre-initialized move generator so that it can easily be re-used
fn perft(board: &mut Board, depth: u8) -> u64 {
    match depth {
        // count 1 for this leaf node
        0 => 1,

        // don't need to make/un-make these moves, just count all leaf nodes from this position
        1 => board.generate_moves().len() as u64,

        // regular case for perft
        d => {
            let mut total = 0;

            for m in board.generate_moves() {
                board.make_move(m);
                total += perft(board, d - 1);
                board.unmake_move();
            }

            total
        }
    }
}

/// Returns identical value to perft function, but prints the perft of every move from starting position
pub fn perft_divide(board: &mut Board, depth: u8) -> u64 {
    if depth == 0 {
        1
    } else {
        let mut total = 0;

        for m in board.generate_moves() {
            board.make_move(m);

            let this_move_total = perft(board, depth - 1);
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
    use crate::board::Board;

    /// Varying positions with expected values from the perft function at a given depth
    ///
    /// Fetched from https://www.chessprogramming.org/Perft_Results, depth is chosen to have expected value ~10 million nodes
    ///
    /// Tuple is in the form (fen, depth, expected)
    const TEST_CASES: [(&str, u8, u64); 3] = [
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
        for (fen, depth, expected) in TEST_CASES {
            let mut b = Board::new(fen);

            assert_eq!(
                perft(&mut b, depth),
                expected,
                "Failed on position {} at depth {}",
                fen,
                depth
            );
        }
    }
}
