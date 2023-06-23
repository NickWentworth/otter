use crate::{
    board::Board,
    move_generator::{Move, MoveGenerator},
};

use super::{evaluate::evaluate, ordering::order_moves, Score, CHECKMATE, DRAW};

// TODO - add in move ordering so that more impactful moves are checked first
// TODO - it seems difficult for the engine to finish out a game, even after it knows there is a checkmate

/// Returns an estimation of the best move by recursively checking opponent's best response is to this move
///
/// Removes invalid move subtrees if worse than an already-evaluated move
///
/// Ensure that the game is not over before calling alpha beta search, as it expects valid moves to be able to be made
pub fn alpha_beta(board: &mut Board, depth: u8) -> (Move, Score) {
    let move_generator = MoveGenerator::new();

    // generate a tuple of moves along with their scores
    let scored_moves = move_generator.generate_moves(board).into_iter().map(|mov| {
        board.make_move(mov);
        let score = -recurse(&move_generator, board, CHECKMATE, -CHECKMATE, depth - 1);
        board.unmake_move();
        (mov, score)
    });

    // find the best move and return it
    scored_moves
        .reduce(|best, next| if next.1 > best.1 { next } else { best })
        .unwrap()
}

/// Recursive step of alpha beta algorithm
fn recurse(
    move_generator: &MoveGenerator,
    board: &mut Board,
    alpha: Score, // represents the worst possible case for the moving side
    beta: Score,  // represents the best possible case for the non-moving side
    depth: u8,
) -> Score {
    // base case - if depth is 0, evaluate the board state
    if depth == 0 {
        // TODO - apply quiescence search to ensure the next response is not about to take our piece
        return evaluate(board);
    }

    // else, generate moves and score them recursively
    let mut moves = move_generator.generate_moves(board);

    // if there are no moves generated, the game is over at this point
    if moves.is_empty() {
        if move_generator.in_check(board) {
            return CHECKMATE;
        } else {
            return DRAW;
        }
    }

    // order the moves based on approximate importance to help remove other bad moves early
    order_moves(&mut moves);

    let mut current_alpha = alpha;

    // go through the moves and find the best score
    for mov in moves {
        // make the move and get the enemy's best response to that move, in terms of our evaluation
        board.make_move(mov);
        let score = -recurse(move_generator, board, -beta, -current_alpha, depth - 1);
        board.unmake_move();

        // if the evaluation for this move is better than the opponent's current best option,
        // they won't allow this to happen, so this move wouldn't even be considered
        if score >= beta {
            return beta;
        }

        // update our current best move
        current_alpha = Score::max(score, current_alpha);
    }

    // return the highest score, the one that we will choose to make
    current_alpha
}
