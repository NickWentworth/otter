use crate::{
    board::Board,
    search::{alpha_beta, mate_in, TranspositionTable},
};

const TT_SIZE: usize = 64;
const SEARCH_DEPTH: u8 = 8;
const MAX_MOVES: usize = 200;

pub struct Engine {
    board: Board,
    table: TranspositionTable,
}

impl Engine {
    /// Generates a new engine, initializing a board and transposition table
    pub fn new(fen: String) -> Engine {
        Engine {
            board: Board::new(fen),
            table: TranspositionTable::new(TT_SIZE),
        }
    }

    /// Plays currently loaded board state to completion
    pub fn play(&mut self) {
        let mut move_count = 0;
        while move_count < MAX_MOVES {
            // check if game is over
            if self.board.generate_moves().is_empty() {
                print!("game over: ");

                if self.board.in_check() {
                    println!("{:?} wins", self.board.active_color().opposite())
                } else {
                    println!("draw");
                }

                break;
            }

            // generate best move
            let (best_move, evaluation) =
                alpha_beta(&mut self.board, &mut self.table, SEARCH_DEPTH);

            // make the move
            self.board.make_move(best_move);
            move_count += 1;
            match mate_in(evaluation) {
                Some(mate) => println!("{} (M{})", best_move, mate),
                None => println!("{} ({})", best_move, evaluation),
            }
        }

        if move_count == MAX_MOVES {
            println!("move limit reached!");
        }

        // self.table.print_stats();
        // println!("{}", self.board);
    }
}
