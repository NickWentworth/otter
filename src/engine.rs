use crate::{
    board::{Board, Magic},
    core::{Color, NUM_COLORS},
    search::Searcher,
};
use std::{io::stdin, thread, time::Duration};

/// Default transposition table size (in MB)
const TT_SIZE: usize = 512;

/// Maximum search time allowed to limit endless searching
const MAX_SEARCH_TIME: Duration = std::time::Duration::from_secs(5);

pub struct Engine {
    board: Board,
    searcher: Searcher,

    // time controls per side
    time: [Duration; NUM_COLORS],
}

impl Engine {
    /// Generates a new engine, initializing a board and transposition table
    pub fn new() -> Engine {
        Engine {
            board: Board::default(),
            searcher: Searcher::new(TT_SIZE),
            time: [Duration::MAX; 2], // start out with no time limit
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
                    println!("id name Otter 1.0");
                    println!("id author Nick Wentworth");
                    println!("uciok");
                }

                Some("ucinewgame") => {
                    // refresh transposition table when a new game is started
                    self.searcher.reset_tt(TT_SIZE);
                }

                Some("isready") => println!("readyok"),

                Some("position") => match tokens.next() {
                    // given a fen string
                    Some("fen") => {
                        // join remaining tokens into the fen string
                        let fen = tokens.collect::<Vec<_>>().join(" ");
                        self.board = Board::new(&fen);
                    }

                    // list of moves made from starting position
                    Some("startpos") => {
                        // next token should be "moves", so ignore it
                        tokens.next();

                        // set board to starting position
                        self.board = Board::default();

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
                    while let Some(param) = tokens.next() {
                        match param {
                            "wtime" => {
                                let time = tokens.next().unwrap().parse().unwrap();
                                self.time[Color::White] = Duration::from_millis(time);
                            }

                            "btime" => {
                                let time = tokens.next().unwrap().parse().unwrap();
                                self.time[Color::Black] = Duration::from_millis(time);
                            }

                            _ => (),
                        }
                    }

                    // calculate how much time we can search for (estimating about 30 moves to be played at this speed)
                    let total_time = self.time[self.board.active_color()];
                    let search_time = Duration::min(total_time / 30, MAX_SEARCH_TIME);

                    // make a clone of the search control after setting it to active
                    let search_control = self.searcher.get_search_control();
                    *search_control.lock().unwrap() = true;

                    // create a thread that will set reference to search control to false after search time is up
                    thread::spawn(move || {
                        thread::sleep(search_time);
                        *search_control.lock().unwrap() = false;
                    });

                    // find best move according to given parameters and print it to stdout
                    match self.searcher.best_move(&mut self.board) {
                        Some((mov, _)) => println!("bestmove {}", mov),
                        None => println!("no moves in this position"),
                    }
                }

                // -------------------- non-uci commands -------------------- //

                // diplay board info
                Some("display") => println!("{}", self.board),

                // display transposition table statistics
                Some("stats") => println!("{}", self.searcher),

                // generate a new set of magic numbers
                Some("generate") => Magic::generate_magics(),

                // display common commands
                Some("help") => {
                    println!();
                    println!("position fen [FEN]\n\tSetup board from fen string\n");
                    println!("go\n\tSearch for best move from current position\n");
                    println!("display\n\tDisplay current position on the board\n");
                }

                // if unable to match a command, do nothing
                Some(_) | None => {
                    println!("Command not recognized, use \"help\" for available commands")
                }
            }
        }
    }
}
