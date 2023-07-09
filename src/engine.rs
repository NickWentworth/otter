use crate::{
    board::Board,
    search::{best_move, mate_in, SearchTT},
};
use std::time::Duration;

const TT_SIZE: usize = 512;
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

    /// Runs in a loop accepting commands from user
    // TODO -  clean this up
    pub fn run(&mut self) {
        loop {
            let mut command = String::new();
            if let Err(err) = std::io::stdin().read_line(&mut command) {
                println!("{}", err);
                continue;
            }

            match command {
                // "fen <fen>"
                // setup board from fen string
                _ if command.starts_with("fen ") => {
                    let fen = command.trim().chars().skip(4).collect::<String>();
                    self.board = Board::new(fen);
                }

                // "search <ms>"
                // prints the best move and evaluation of the position, with allotted time to search
                _ if command.starts_with("search ") => {
                    let time = command
                        .trim()
                        .chars()
                        .skip(7)
                        .collect::<String>()
                        .parse::<u64>();

                    if let Err(err) = time {
                        println!("Error parsing time: {}", err.to_string());
                        continue;
                    }

                    match best_move(
                        &mut self.board,
                        &mut self.table,
                        Duration::from_millis(time.unwrap()),
                    ) {
                        Some((best_move, evaluation)) => {
                            println!("Move: {}\nEvaluation: {}", best_move, evaluation);
                        }

                        None => println!("No valid moves from this position"),
                    }
                }

                // "move <algebraic notation>"
                // makes a move from the current position, if legal
                _ if command.starts_with("move ") => {
                    let move_string = command.trim().chars().skip(5).collect::<String>();

                    match self
                        .board
                        .generate_moves()
                        .into_iter()
                        .find(|mov| mov.to_string() == move_string)
                    {
                        Some(mov) => self.board.make_move(mov),
                        None => println!("{} is not a valid move", move_string),
                    }
                }

                // "play"
                // plays the game to completion from current position
                _ if command.starts_with("play") => self.play(),

                // "display"
                // prints current board state
                _ if command.starts_with("display") => println!("{}", self.board),

                // catch for invalid commands
                _ => println!("{} is not a valid command!", command),
            }
        }
    }

    /// Plays currently loaded board state to completion
    fn play(&mut self) {
        for _ in 0..MAX_MOVES {
            // check for draws
            if self.board.is_drawable() {
                println!("draw");
                break;
            }

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
