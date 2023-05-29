use std::fmt::Display;

use crate::{
    fen::{check_valid_fen, DEFAULT_FEN},
    move_generator::Move,
    types::{Bitboard, Color, Piece, Square, NUM_COLORS, NUM_PIECES},
    utility::{pop_msb_1, square_from_algebraic, MSB_BOARD},
};

/// Variables related to conditions of the game
struct GameState {
    current_turn: Color,
    white_king_castle: bool, // castling availability for each color and king/queen side
    white_queen_castle: bool,
    black_king_castle: bool,
    black_queen_castle: bool,
    en_passant_square: Option<Square>,
    halfmove: u32, // halfmove counter, incremented after each color's move
    fullmove: u32, // fullmove counter, only incremented after black's move
}

/// Overall representation of a chess game
pub struct Board {
    pieces: [Bitboard; NUM_PIECES],
    colors: [Bitboard; NUM_COLORS],
    game_state: GameState,
}

impl Board {
    /// Generates a new `Board` from a given FEN string
    ///
    /// The FEN string is validated, if invalid the board is set to the start state of the chess game
    pub fn new(fen: String) -> Board {
        // check if the given string is valid
        let fen_parts: Vec<String> = if check_valid_fen(&fen) {
            fen
        } else {
            // if not, just use the default fen string for now
            println!("Invalid FEN! Reverting to starting board state.");
            DEFAULT_FEN.to_string()
        }
        .split(" ")
        .map(|s| s.to_string())
        .collect();

        // build the bitboards for the struct
        let mut pieces: [Bitboard; NUM_PIECES] = [0; NUM_PIECES];
        let mut colors: [Bitboard; NUM_COLORS] = [0; NUM_COLORS];

        // index bitboard to be AND-ed with piece/color bitboards, bitwise looks like 1000...0000
        // instead of being incremented, this is right-shifted to move the 1 over
        let mut index: Bitboard = MSB_BOARD;

        for symbol in fen_parts[0].chars() {
            match symbol {
                // number in FEN means # of empty spaces
                n if n.is_ascii_digit() => {
                    // increment the index by number of empty spaces
                    index >>= n.to_digit(10).unwrap() as usize;
                }

                // slash indicates the next column, but we don't have to do anything here
                '/' => continue,

                // else, try to parse character as a piece
                c => {
                    // change color bitboards based on case
                    if c.is_ascii_uppercase() {
                        colors[Color::White] |= index;
                    } else {
                        colors[Color::Black] |= index;
                    }

                    // change piece bitboards based on letter
                    match c {
                        'P' | 'p' => pieces[Piece::Pawn] |= index,
                        'N' | 'n' => pieces[Piece::Knight] |= index,
                        'B' | 'b' => pieces[Piece::Bishop] |= index,
                        'R' | 'r' => pieces[Piece::Rook] |= index,
                        'Q' | 'q' => pieces[Piece::Queen] |= index,
                        'K' | 'k' => pieces[Piece::King] |= index,
                        _ => panic!("invalid symbol in fen string"),
                    }

                    // finally increment index
                    index >>= 1;
                }
            }
        }

        Board {
            pieces,
            colors,
            game_state: GameState {
                current_turn: if fen_parts[1] == "w" {
                    Color::White
                } else {
                    Color::Black
                },
                white_king_castle: fen_parts[2].contains('K'),
                white_queen_castle: fen_parts[2].contains('Q'),
                black_king_castle: fen_parts[2].contains('k'),
                black_queen_castle: fen_parts[2].contains('q'),
                en_passant_square: square_from_algebraic(&fen_parts[3]), // will handle correctly if passed "-"
                halfmove: fen_parts[4].parse().unwrap(),
                fullmove: fen_parts[5].parse().unwrap(),
            },
        }
    }

    /// Makes the given move and updates game state accordingly
    ///
    /// Assumes `m` is a valid and legal move
    pub fn make_move(&mut self, m: &Move) {
        // store locally because of borrow checker
        let moving_color = self.active_color();

        // bitwise XOR with a board with 1's at both from and to squares for colors
        // the from location will be set to 0 and the to location will be set to 1
        // other locations will be left unchanged (0^0 = 0, 1^0 = 1)
        let move_board = (MSB_BOARD >> m.from) | (MSB_BOARD >> m.to);

        // apply the changes
        self.colors[moving_color] ^= move_board;
        self.pieces[m.piece] ^= move_board;

        // update game state
        // TODO - more updating needs to be done after further move flags are added
        if moving_color == Color::Black {
            self.game_state.fullmove += 1;
        }

        self.game_state.current_turn = self.game_state.current_turn.opposite();
    }

    /// Generates a bitboard of pieces matching the given type that can move this turn
    pub fn active_piece_board(&self, piece: Piece) -> Bitboard {
        self.pieces[piece] & self.active_color_board()
    }

    /// Returns a copy of the bitboard of the current moving color
    pub fn active_color_board(&self) -> Bitboard {
        self.colors[self.game_state.current_turn]
    }

    /// Returns a copy of the bitboard of the current non-moving color
    pub fn inactive_color_board(&self) -> Bitboard {
        self.colors[self.game_state.current_turn.opposite()]
    }

    /// Returns the color enum of the current moving color
    pub fn active_color(&self) -> Color {
        self.game_state.current_turn
    }

    /// Generates a bitboard containing the en passant square or an empty board if there is no square
    pub fn en_passant_square(&self) -> Bitboard {
        match self.game_state.en_passant_square {
            Some(square) => MSB_BOARD >> square,
            None => 0,
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Color::*;
        use Piece::*;

        let mut chars = ['.'; 64];

        // generate array of characters representing pieces
        for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
            let mut piece_board = self.pieces[piece];

            while piece_board != 0 {
                let square = pop_msb_1(&mut piece_board) as usize;
                let position = MSB_BOARD >> square;

                // match the character at this square to a piece on the board
                chars[square] = match piece {
                    Pawn => 'P',
                    Knight => 'N',
                    Bishop => 'B',
                    Rook => 'R',
                    Queen => 'Q',
                    King => 'K',
                };

                // if piece is black, lowercase it
                if position & self.colors[White] == 0 {
                    chars[square] = chars[square].to_ascii_lowercase();
                }
            }
        }

        // build the board string from the character array
        let mut output = String::new();
        let mut index = 0;

        for symbol in chars {
            if index % 8 == 0 {
                output.push('\n');
                output.push_str(&format!("{}   ", 8 - index / 8));
            }

            output.push(symbol);
            output.push(' ');
            index += 1;
        }

        output.push_str("\n\n    a b c d e f g h\n");

        // add some related game state info
        let move_info = format!(
            "\nTurn: {} | Fullmove: {} | Halfmove: {}\n",
            match self.game_state.current_turn {
                White => "White",
                Black => "Black",
            },
            self.game_state.fullmove,
            self.game_state.halfmove,
        );
        output.push_str(&move_info);

        let castle_info = format!(
            "Castling availability: {} {} {} {}\n",
            if self.game_state.white_king_castle {
                "K"
            } else {
                "-"
            },
            if self.game_state.white_queen_castle {
                "Q"
            } else {
                "-"
            },
            if self.game_state.black_king_castle {
                "k"
            } else {
                "-"
            },
            if self.game_state.black_queen_castle {
                "q"
            } else {
                "-"
            },
        );
        output.push_str(&castle_info);

        // and write to the formatter
        write!(f, "{}", output)
    }
}
