use crate::{
    board::Board,
    search::{best_move, mate_in, SearchTT},
};

const TT_SIZE: usize = 64;
const SEARCH_DEPTH: u8 = 8;
const MAX_MOVES: usize = 200;

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
            match best_move(&mut self.board, &mut self.table, SEARCH_DEPTH) {
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
