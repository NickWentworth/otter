use crate::{
    board::Board,
    search::{best_move, mate_in, SearchTT},
};
use std::time::Duration;

const TT_SIZE: usize = 64;
const MAX_MOVES: usize = 200;

// TODO - store some current time for each side and use that to determine search time
const SEARCH_TIME: Duration = std::time::Duration::from_secs(1);

pub struct Engine {
    board: Board,
    table: SearchTT,
}

impl Engine {
    /// Generates a new engine, initializing a board and transposition table
    pub fn new(fen: String) -> Engine {
        Engine {
            board: Board::new(fen),
            table: SearchTT::new(TT_SIZE),
        }
    }

    /// Plays currently loaded board state to completion
    pub fn play(&mut self) {
        for _ in 0..MAX_MOVES {
            // generate best move
            match best_move(&mut self.board, &mut self.table, SEARCH_TIME) {
                Some((best_move, evaluation)) => {
                    // make the move
                    self.board.make_move(best_move);

                    match mate_in(evaluation) {
                        Some(mate) => println!("{} (M{})", best_move, mate),
                        None => println!("{} ({})", best_move, evaluation),
                    }
                }

                None => {
                    // if no moves can be generated, game is over
                    print!("game over: ");

                    if self.board.in_check() {
                        println!("{:?} wins", self.board.active_color().opposite())
                    } else {
                        println!("draw");
                    }

                    break;
                }
            }
        }

        // self.table.print_stats();
        // println!("{}", self.board);
    }
}
