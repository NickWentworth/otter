use crate::board::{Board, Move};

use super::{
    evaluate::evaluate, ordering::order_moves, tt::TranspositionTable, Score, CHECKMATE, DRAW,
};

/// Value where a score can still be considered a checkmate
const CHECKMATE_THRESHOLD: Score = -CHECKMATE / 2;

#[derive(Clone, Copy, Default)]
enum ScoreLimit {
    #[default]
    Exact, // an exact score value has been found for this position
    Alpha, // an upper bound has been found for this position
    Beta,  // a lower bound has been found for this position
}

#[derive(Clone, Copy, Default)]
pub struct ScoreData {
    score: Score,
    depth: u8,
    flag: ScoreLimit, // denotes the bounds of the stored score
}

/// Returns an estimation of the best move by recursively checking opponent's best response is to this move
///
/// Removes invalid move subtrees if worse than an already-evaluated move
///
/// Ensure that the game is not over before calling alpha beta search, as it expects valid moves to be able to be made
pub fn alpha_beta(
    board: &mut Board,
    table: &mut TranspositionTable<ScoreData>,
    depth: u8,
) -> (Move, Score) {
    // generate a tuple of moves along with their scores
    let scored_moves = board.generate_moves().into_iter().map(|mov| {
        board.make_move(mov);
        let score = -recurse(board, table, CHECKMATE, -CHECKMATE, depth - 1, 1);
        board.unmake_move();
        (mov, score)
    });

    // find the best move
    scored_moves
        .reduce(|best, next| if next.1 > best.1 { next } else { best })
        .unwrap()
}

/// Recursive step of alpha beta algorithm
fn recurse(
    board: &mut Board,
    table: &mut TranspositionTable<ScoreData>,
    alpha: Score, // represents the worst possible case for the moving side
    beta: Score,  // represents the best possible case for the non-moving side
    depth: u8,
    ply: u8,
) -> Score {
    use ScoreLimit::*;

    // TODO - handle 3-fold repetition draws

    // base case - if depth is 0, evaluate the board state
    if depth == 0 {
        return quiesce(board, alpha, beta);
    }

    // check if this position has already been evaluated and is stored in the transposition table
    if let Some(data) = table.get(board.zobrist()) {
        // only consider positions searched to at least the current depth
        if data.depth >= depth {
            match data.flag {
                // if exact, we can just return the score
                Exact => return convert_mate(data.score, ply),

                // if alpha, ensure that the upper bound given is within our limits for upper bound
                Alpha => {
                    if data.score <= alpha {
                        return convert_mate(data.score, ply);
                    }
                }

                // if beta, ensure that the lower bound given is within our limits for lower bound
                Beta => {
                    if data.score >= beta {
                        return convert_mate(data.score, ply);
                    }
                }
            }
        }
    }

    // else, generate moves and score them recursively
    let mut moves = board.generate_moves();

    // if there are no moves generated, the game is over at this point
    if moves.is_empty() {
        if board.in_check() {
            // add ply to checkmate score to sort faster (lower ply) checkmates higher
            return CHECKMATE + (ply as Score);
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
        let score = -recurse(board, table, -beta, -current_alpha, depth - 1, ply + 1);
        board.unmake_move();

        // if the evaluation for this move is better than the opponent's current best option,
        // they won't allow this to happen, so this move wouldn't even be considered
        if score >= beta {
            // add this board configuration into the transposition table
            table.insert(
                board.zobrist(),
                ScoreData {
                    score,
                    depth,
                    flag: Beta,
                },
            );

            return beta;
        }

        // update our current best move
        current_alpha = Score::max(score, current_alpha);
    }

    // add this board configuration into the transposition table
    table.insert(
        board.zobrist(),
        ScoreData {
            score: current_alpha,
            depth,
            flag: match alpha == current_alpha {
                true => Exact,  // an improvement has been made to our upper bound
                false => Alpha, // all that we know is that this position has an upper bound
            },
        },
    );

    // return the highest score, the one that we will choose to make
    current_alpha
}

/// Final step of alpha beta search, before evaluation we want to ensure that our moved piece is not about to be captured
///
/// Searches down all capture-only paths until a quiet position is found for each
fn quiesce(
    board: &mut Board,
    alpha: Score, // represents the worst possible case for the moving side
    beta: Score,  // represents the best possible case for the non-moving side
) -> Score {
    // first get the current board evaluation
    let current_score = evaluate(board);

    // if the score of this board is higher than the best guarantee (worse for the previous color), they wouldn't make this capture
    if current_score >= beta {
        return beta;
    }

    // otherwise, the best case for the active color is max between this and previous best case
    let mut current_alpha = Score::max(current_score, alpha);

    let mut captures = board.generate_captures();
    order_moves(&mut captures);

    // this is same as alpha beta search
    for mov in captures {
        board.make_move(mov);
        let score = -quiesce(board, -beta, -current_alpha);
        board.unmake_move();

        if score >= beta {
            return beta;
        }

        current_alpha = Score::max(score, current_alpha);
    }

    current_alpha
}

/// Converts a tt-fetched checkmate to a ply-offset checkmate for correct ordering of mating scores based on ply
fn convert_mate(score: Score, ply: u8) -> Score {
    if score > CHECKMATE_THRESHOLD {
        score - (ply as Score)
    } else if score < -CHECKMATE_THRESHOLD {
        score + (ply as Score)
    } else {
        score
    }
}
