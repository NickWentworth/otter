use crate::{
    board::Board,
    core::Color,
    move_generator::{Move, MoveGenerator},
};

use super::{evaluate::evaluate, Score, CHECKMATE_BLACK, CHECKMATE_WHITE, DRAW};

/// Recursively chooses moves based on the best other player's best response to the given move
// TODO - its a bit ugly to map each move to a score in this way, also need to factor in checkmates and draws
// TODO - fix search function now that advantage is based on moving color, not always + for white, - for black
pub fn minimax(board: &mut Board, depth: u8) -> Move {
    use Color::*;

    let move_generator = MoveGenerator::new();

    let moves = move_generator.generate_moves(board);

    let mut best_move = moves[0];
    match board.active_color() {
        White => {
            // start out with lowest number worst case
            let mut best_score = Score::MIN;

            for mov in moves {
                board.make_move(mov);

                let score = minimax_step(&move_generator, board, depth - 1);
                if score > best_score {
                    best_move = mov;
                    best_score = score;
                }

                board.unmake_move();
            }
        }
        Black => {
            // start out with highest number worst case
            let mut best_score = Score::MAX;

            for mov in moves {
                board.make_move(mov);

                let score = minimax_step(&move_generator, board, depth - 1);
                if score < best_score {
                    best_move = mov;
                    best_score = score;
                }

                board.unmake_move();
            }
        }
    }

    best_move
}

fn minimax_step(move_generator: &MoveGenerator, board: &mut Board, depth: u8) -> Score {
    use Color::*;

    if depth == 0 {
        // base case: evaluate the board at this state
        evaluate(board)
    } else {
        let active_color = board.active_color();

        // for each legal move, find the score of the opponent's best response to this move
        let scores = move_generator.generate_moves(board).into_iter().map(|mov| {
            board.make_move(mov);
            let score = minimax_step(move_generator, board, depth - 1);
            board.unmake_move();
            score
        });

        // based on the scores (and active color), choose a move to make
        let best_score = match active_color {
            White => scores.max(),
            Black => scores.min(),
        };

        match best_score {
            // if there exists a best score, return it
            Some(score) => score,

            // if no moves can be made, the moving player is either in check or stalemate
            None => match (move_generator.in_check(board), active_color) {
                // if a color has no moves and is in check, then the opposite color has a checkmate
                (true, White) => CHECKMATE_BLACK,
                (true, Black) => CHECKMATE_WHITE,

                // if not in check with no moves, we are in a stalemate
                (false, _) => DRAW,
            },
        }
    }
}
