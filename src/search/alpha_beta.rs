use crate::board::{Board, Move};

use super::{evaluate::evaluate, ordering::order_moves, tt::TranspositionTable, Score};

// Scores pertaining to different constant cases
const INFINITY: Score = 30000;
const CHECKMATE: Score = 25000;
const CHECKMATE_THRESHOLD: Score = 20000; // values above this can be considered "mate in _"
const DRAW: Score = 0;

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
pub fn best_move(
    board: &mut Board,
    table: &mut TranspositionTable<ScoreData>,
    depth: u8,
) -> Option<(Move, Score)> {
    // generate a tuple of moves along with their scores and find the max
    board
        .generate_moves()
        .into_iter()
        .map(|mov| {
            board.make_move(mov);
            let score = -alpha_beta(board, table, -INFINITY, INFINITY, depth - 1, 1);
            board.unmake_move();
            (mov, score)
        })
        .max_by_key(|(_, score)| score.clone()) // max by the score value
}

/// Recursive step of alpha beta algorithm
fn alpha_beta(
    board: &mut Board,
    table: &mut TranspositionTable<ScoreData>,
    mut alpha: Score, // upper bound for the moving side (known best case)
    beta: Score,      // lower bound for the moving side (known worst case)
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
            // convert the score to the proper format for checkmates
            let converted_score = convert_score_get(data.score, ply);

            match data.flag {
                // if exact, we can just return the score
                Exact => return converted_score,

                // if alpha, ensure that the upper bound given is within our limits for upper bound
                Alpha => {
                    if converted_score <= alpha {
                        return converted_score;
                    }
                }

                // if beta, ensure that the lower bound given is within our limits for lower bound
                Beta => {
                    if converted_score >= beta {
                        return converted_score;
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
            // add ply to checkmate score to denote "mate in (ply)" from the initial position
            return -CHECKMATE + (ply as Score);
        } else {
            return DRAW;
        }
    }

    // order the moves based on approximate importance to help remove other bad moves early
    order_moves(&mut moves);

    // keep track of if this position's score is an upper bound or exact
    let mut flag = Alpha;

    // go through the moves and find the best score
    for mov in moves {
        // make the move and get the enemy's best response to that move, in terms of our evaluation
        board.make_move(mov);
        let score = -alpha_beta(board, table, -beta, -alpha, depth - 1, ply + 1);
        board.unmake_move();

        // if the evaluation for this move is better than the opponent's current best option,
        // they won't allow this to happen, so this move wouldn't even be considered
        if score >= beta {
            // add this board configuration into the transposition table
            table.insert(
                board.zobrist(),
                ScoreData {
                    score: convert_score_insert(beta, ply),
                    depth,
                    flag: Beta,
                },
            );

            return beta;
        }

        // update our current best move
        if score > alpha {
            flag = Exact; // we now have an exact move score
            alpha = score; // and should update the currently known best move
        }
    }

    // add this board configuration into the transposition table
    table.insert(
        board.zobrist(),
        ScoreData {
            score: convert_score_insert(alpha, ply),
            depth,
            flag,
        },
    );

    // return our best case score value
    alpha
}

/// Final step of alpha beta search, before evaluation we want to ensure that our moved piece is not about to be captured
///
/// Searches down all capture-only paths until a quiet position is found for each
fn quiesce(
    board: &mut Board,
    mut alpha: Score, // represents the worst possible case for the moving side
    beta: Score,      // represents the best possible case for the non-moving side
) -> Score {
    // first get the current board evaluation
    let current_score = evaluate(board);

    // if the score of this board is higher than the best guarantee (worse for the previous color), they wouldn't make this capture
    if current_score >= beta {
        return beta;
    }

    // otherwise, the best case for the active color is max between this and previous best case
    alpha = Score::max(alpha, current_score);

    let mut captures = board.generate_captures();
    order_moves(&mut captures);

    // this is same as alpha beta search
    for mov in captures {
        board.make_move(mov);
        let score = -quiesce(board, -beta, -alpha);
        board.unmake_move();

        if score >= beta {
            return beta;
        }

        alpha = Score::max(alpha, score);
    }

    alpha
}

// checkmates are stored in the transposition table as "mate in _ from this position" scores
// to allow for the same transposition to be used, we must convert into the correct form when reading/writing

fn convert_score_get(score: Score, ply: u8) -> Score {
    if score > CHECKMATE_THRESHOLD {
        score - (ply as Score)
    } else if score < -CHECKMATE_THRESHOLD {
        score + (ply as Score)
    } else {
        score
    }
}

fn convert_score_insert(score: Score, ply: u8) -> Score {
    if score > CHECKMATE_THRESHOLD {
        score + (ply as Score)
    } else if score < -CHECKMATE_THRESHOLD {
        score - (ply as Score)
    } else {
        score
    }
}

/// Returns how many turns a mate will occur in, if applicable
pub fn mate_in(score: Score) -> Option<Score> {
    match score.abs() > CHECKMATE_THRESHOLD {
        true => Some(CHECKMATE - score.abs()),
        false => None,
    }
}
