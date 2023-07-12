use crate::{
    board::{Board, DEFAULT_FEN},
    search::{best_move, SearchTT},
};
use std::{io::stdin, time::Duration};

const TT_SIZE: usize = 512;

// TODO - store some current time for each side and use that to determine search time
const SEARCH_TIME: Duration = std::time::Duration::from_millis(1000);

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

    /// Starts the engine, which communicates with a game client via Universal Chess Interface (UCI)
    ///
    /// Useful info about UCI here: https://gist.github.com/aliostad/f4470274f39d29b788c1b09519e67372
    pub fn uci(&mut self) {
        loop {
            // get next command from stdin
            let mut command = String::new();
            stdin().read_line(&mut command).unwrap();

            // start reading through the tokens in the command
            let mut tokens = command.trim().split(' ');

            match tokens.next() {
                Some("uci") => {
                    // print out some info about the engine
                    println!("id name Engine Name");
                    println!("id author Nick Wentworth");
                    println!("uciok");
                }

                Some("ucinewgame") => {
                    // refresh transposition table when a new game is started
                    self.table = SearchTT::new(TT_SIZE);
                }

                Some("isready") => println!("readyok"),

                Some("position") => match tokens.next() {
                    // given a fen string
                    Some("fen") => {
                        // join remaining tokens into the fen string
                        let fen = tokens.collect::<Vec<_>>().join(" ");
                        self.board = Board::new(fen);
                    }

                    // list of moves made from starting position
                    Some("startpos") => {
                        // next token should be "moves", so ignore it
                        tokens.next();

                        // set board to starting position
                        self.board = Board::new(DEFAULT_FEN.to_string());

                        while let Some(move_string) = tokens.next() {
                            // try to find this move string from all current legal move strings
                            match self
                                .board
                                .generate_moves()
                                .into_iter()
                                .find(|mov| mov.to_string() == move_string)
                            {
                                Some(legal_move) => self.board.make_move(legal_move),
                                None => println!("{} is not a legal move!", move_string),
                            }
                        }
                    }

                    _ => (),
                },

                Some("go") => {
                    // TODO - handle parameters like "wtime", "btime", etc.

                    // find best move according to given parameters and print it to stdout
                    let best_move = best_move(&mut self.board, &mut self.table, SEARCH_TIME);
                    if let Some((mov, _)) = best_move {
                        println!("bestmove {}", mov);
                    }
                }

                // -------------------- non-uci commands -------------------- //
                Some("display") => println!("{}", self.board),

                // if unable to match a command, do nothing
                Some(_) => (),
                None => (),
            }
        }
    }
}
