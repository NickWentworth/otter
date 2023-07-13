use crate::board::{Board, Move};
use std::time::{Duration, Instant};

use super::{evaluate::evaluate, ordering::order_moves, tt::TranspositionTable, Score};

// Scores pertaining to different constant cases
const INFINITY: Score = 30000;
const CHECKMATE: Score = 25000;
const CHECKMATE_THRESHOLD: Score = 20000; // values above this can be considered "mate in _"
const DRAW: Score = 0;

/// Maximum depth allowed to be searched to
const MAX_DEPTH: u8 = u8::MAX;

/// Transposition table used for searching, stores required data about scoring a position
pub type SearchTT = TranspositionTable<ScoreData>;

#[derive(Clone, Copy, Default)]
enum ScoreLimit {
    #[default]
    Exact, // an exact score value has been found for this position
    Alpha, // an upper bound has been found for this position
    Beta,  // a lower bound has been found for this position
}

// TODO - move struct being here changes size of this from 4 bytes to 48 bytes, need to pack moves into a smaller struct
#[derive(Clone, Copy, Default)]
pub struct ScoreData {
    score: Score,
    depth: u8,
    flag: ScoreLimit,        // denotes the bounds of the stored score
    best_move: Option<Move>, // if found, the current best move from this position
}

pub struct Searcher {
    table: SearchTT,
}

impl Searcher {
    pub fn new(tt_size: usize) -> Searcher {
        Searcher {
            table: SearchTT::new(tt_size),
        }
    }

    pub fn reset_tt(&mut self, tt_size: usize) {
        self.table = SearchTT::new(tt_size);
    }

    /// Returns an estimation of the best move by recursively checking opponent's best response is to this move
    pub fn best_move(&mut self, board: &mut Board, search_time: Duration) -> Option<(Move, Score)> {
        let mut best: Option<(Move, i16)> = None;
        let used_time = Instant::now();

        // iterative deepening - keep incrementing depth until an alloted search time is used up
        for depth in 1..MAX_DEPTH {
            // break if allotted search time was reached
            if used_time.elapsed() >= search_time {
                break;
            }

            // generate a tuple of moves along with their scores and find the max
            best = board
                .generate_moves()
                .into_iter()
                .map(|mov| {
                    board.make_move(mov);
                    let score = -self.alpha_beta(board, -INFINITY, INFINITY, depth, 1);
                    board.unmake_move();
                    (mov, score)
                })
                .max_by_key(|(_, score)| score.clone()); // max by the score value

            // leave early if we found a forced mate sequence
            if let Some((_, score)) = best {
                if score.abs() > CHECKMATE_THRESHOLD {
                    break;
                }
            }
        }

        best
    }

    /// Recursive step of alpha beta algorithm
    fn alpha_beta(
        &mut self,
        board: &mut Board,
        mut alpha: Score,
        beta: Score,
        depth: u8,
        ply: u8,
    ) -> Score {
        use ScoreLimit::*;

        // TODO - this may not always properly handle draws, as transposition table sees repetitions 1, 2, and 3 as the same hash
        if board.is_drawable() {
            return DRAW;
        }

        // base case - if depth is 0, evaluate the board state
        if depth == 0 {
            return Self::quiesce(board, alpha, beta);
        }

        // check if this position has already been evaluated and is stored in the transposition table
        let best_move = match self.table.get(board.zobrist()) {
            Some(data) => {
                // only consider scores from positions searched to at least the current depth
                if data.depth >= depth {
                    // convert the score to the proper format for checkmates
                    let converted_score = Self::convert_score_get(data.score, ply);

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

                // if the table stored a position but couldn't be used, at least order the best move first
                data.best_move
            }

            // no data to go off from
            None => None,
        };

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
        order_moves(&mut moves, best_move);

        // keep track of if this position's score is an upper bound or exact
        let mut flag = Alpha;
        let mut best_move = None;

        // go through the moves and find the best score
        for mov in moves {
            // make the move and get the enemy's best response to that move, in terms of our evaluation
            board.make_move(mov);
            let score = -self.alpha_beta(board, -beta, -alpha, depth - 1, ply + 1);
            board.unmake_move();

            // if the evaluation for this move is better than the opponent's current best option,
            // they won't allow this to happen, so this move wouldn't even be considered
            if score >= beta {
                // add this board configuration into the transposition table
                self.table.insert(
                    board.zobrist(),
                    ScoreData {
                        score: Self::convert_score_insert(beta, ply),
                        depth,
                        flag: Beta,
                        best_move: Some(mov),
                    },
                );

                return beta;
            }

            // update our current best move
            if score > alpha {
                flag = Exact; // we now have an exact move score
                alpha = score; // update the currently known best move
                best_move = Some(mov); // and store this move as best
            }
        }

        // add this board configuration into the transposition table
        self.table.insert(
            board.zobrist(),
            ScoreData {
                score: Self::convert_score_insert(alpha, ply),
                depth,
                flag,
                best_move,
            },
        );

        // return our best case score value
        alpha
    }

    /// Final step of alpha beta search, before evaluation we want to ensure that our moved piece is not about to be captured
    ///
    /// Searches down all capture-only paths until a quiet position is found for each
    fn quiesce(board: &mut Board, mut alpha: Score, beta: Score) -> Score {
        // first get the current board evaluation
        let current_score = evaluate(board);

        // if the score of this board is higher than the best guarantee (worse for the previous color), they wouldn't make this capture
        if current_score >= beta {
            return beta;
        }

        // otherwise, the best case for the active color is max between this and previous best case
        alpha = Score::max(alpha, current_score);

        let mut captures = board.generate_captures();
        order_moves(&mut captures, None);

        // this is same as alpha beta search
        for mov in captures {
            board.make_move(mov);
            let score = -Self::quiesce(board, -beta, -alpha);
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
}
