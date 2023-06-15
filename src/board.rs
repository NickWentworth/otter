use crate::{
    core::{
        Bitboard, Color, Piece, Square, ALGEBRAIC_NOTATION, ALL_PIECES, BOARD_SIZE, NUM_COLORS,
        NUM_PIECES,
    },
    move_generator::{Move, MoveFlag},
};
use std::fmt::Display;

mod fen;
mod position;

use fen::{check_valid_fen, DEFAULT_FEN};
pub use position::Position;

const INITIAL_KINGSIDE_ROOK_SQUARES: [Square; NUM_COLORS] = [63, 7];
const INITIAL_QUEENSIDE_ROOK_SQUARES: [Square; NUM_COLORS] = [56, 0];

/// Variables related to conditions of the game
#[derive(Clone)]
struct GameState {
    current_turn: Color,
    king_castle: [bool; NUM_COLORS], // castling availability for each color and king/queen side
    queen_castle: [bool; NUM_COLORS],
    en_passant_square: Option<Square>,
    halfmove: u32, // halfmove counter, incremented after each color's move
    fullmove: u32, // fullmove counter, only incremented after black's move
}

/// Overall representation of a chess game
pub struct Board {
    pieces: [Bitboard; NUM_PIECES],
    colors: [Bitboard; NUM_COLORS],
    game_state: GameState,

    // stack containing moves and matching info needed to unmake the previously made move
    history: Vec<(Move, GameState)>,
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
        let mut pieces: [Bitboard; NUM_PIECES] = [Bitboard::EMPTY; NUM_PIECES];
        let mut colors: [Bitboard; NUM_COLORS] = [Bitboard::EMPTY; NUM_COLORS];

        // index bitboard to be AND-ed with piece/color bitboards, bitwise looks like 1000...0000
        // instead of being incremented, this is right-shifted to move the 1 over
        let mut index: Bitboard = Bitboard::MSB;

        for symbol in fen_parts[0].chars() {
            match symbol {
                // number in FEN means # of empty spaces
                n if n.is_ascii_digit() => {
                    // increment the index by number of empty spaces
                    index >>= n.to_digit(10).unwrap() as Square;
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
                    index >>= 1 as Square;
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
                king_castle: [fen_parts[2].contains('K'), fen_parts[2].contains('k')],
                queen_castle: [fen_parts[2].contains('Q'), fen_parts[2].contains('q')],
                en_passant_square: ALGEBRAIC_NOTATION.iter().position(|&s| s == fen_parts[3]),
                halfmove: fen_parts[4].parse().unwrap(),
                fullmove: fen_parts[5].parse().unwrap(),
            },
            history: Vec::new(),
        }
    }

    /// Makes the given move and updates game state accordingly
    ///
    /// Assumes `m` is a valid and legal move
    pub fn make_move(&mut self, m: &Move) {
        // push move and current game state to stack
        self.history.push((m.clone(), self.game_state.clone()));

        // store locally because of borrow checker
        let moving_color = self.game_state.current_turn;

        // make the move, just set move bits m.from -> m.to
        self.colors[moving_color].set_bit_at(m.from, false);
        self.colors[moving_color].set_bit_at(m.to, true);
        self.pieces[m.piece].set_bit_at(m.from, false);
        self.pieces[m.piece].set_bit_at(m.to, true);

        // apply the unique move flag cases
        use MoveFlag::*;
        match m.flag {
            // nothing more to do
            Quiet => (),

            // need to change to piece from pawn to the promoted one
            Promotion(promoted_piece) => {
                self.pieces[m.piece].set_bit_at(m.to, false);
                self.pieces[promoted_piece].set_bit_at(m.to, true);
            }

            // need to remove the opposing color's piece
            Capture(captured_piece) => {
                self.colors[moving_color.opposite()].set_bit_at(m.to, false);

                // only set captured piece bit false if it is different than the moving piece, else both pieces will disappear
                if m.piece != captured_piece {
                    self.pieces[captured_piece].set_bit_at(m.to, false);
                }

                // update castling rights if taking opposing rook
                if captured_piece == Piece::Rook {
                    if self.game_state.king_castle[moving_color.opposite()]
                        && INITIAL_KINGSIDE_ROOK_SQUARES[moving_color.opposite()] == m.to
                    {
                        self.game_state.king_castle[moving_color.opposite()] = false;
                    }

                    if self.game_state.queen_castle[moving_color.opposite()]
                        && INITIAL_QUEENSIDE_ROOK_SQUARES[moving_color.opposite()] == m.to
                    {
                        self.game_state.queen_castle[moving_color.opposite()] = false;
                    }
                }
            }

            // combination of capture and promotion
            CapturePromotion(captured_piece, promoted_piece) => {
                // do promotion changes
                self.pieces[m.piece].set_bit_at(m.to, false);
                self.pieces[promoted_piece].set_bit_at(m.to, true);

                // do capture changes
                self.colors[moving_color.opposite()].set_bit_at(m.to, false);

                // same as capture flag, don't want both pieces to disappear if capturing the same piece that is being promoted to
                if captured_piece != promoted_piece {
                    self.pieces[captured_piece].set_bit_at(m.to, false);
                }

                // update castling rights if taking opposing rook
                if captured_piece == Piece::Rook {
                    if self.game_state.king_castle[moving_color.opposite()]
                        && INITIAL_KINGSIDE_ROOK_SQUARES[moving_color.opposite()] == m.to
                    {
                        self.game_state.king_castle[moving_color.opposite()] = false;
                    }

                    if self.game_state.queen_castle[moving_color.opposite()]
                        && INITIAL_QUEENSIDE_ROOK_SQUARES[moving_color.opposite()] == m.to
                    {
                        self.game_state.queen_castle[moving_color.opposite()] = false;
                    }
                }
            }

            // set the en passant square later on
            PawnDoubleMove(_) => (),

            // en passant square is in a different spot than a regular capture
            EnPassantCapture(enemy_pawn_square) => {
                self.colors[moving_color.opposite()].set_bit_at(enemy_pawn_square, false);
                self.pieces[Piece::Pawn].set_bit_at(enemy_pawn_square, false);
            }

            // move the rook to the correct square and change castling rights
            KingCastle => {
                // move rook (calculated from m.to)
                self.colors[moving_color].set_bit_at(m.to + 1, false);
                self.colors[moving_color].set_bit_at(m.to - 1, true);

                self.pieces[Piece::Rook].set_bit_at(m.to + 1, false);
                self.pieces[Piece::Rook].set_bit_at(m.to - 1, true);

                // finally remove castling rights
                self.game_state.king_castle[moving_color] = false;
                self.game_state.queen_castle[moving_color] = false;
            }

            // move the rook to the correct square and change castling rights
            QueenCastle => {
                // move rook (calculated from m.to)
                self.colors[moving_color].set_bit_at(m.to - 2, false);
                self.colors[moving_color].set_bit_at(m.to + 1, true);

                self.pieces[Piece::Rook].set_bit_at(m.to - 2, false);
                self.pieces[Piece::Rook].set_bit_at(m.to + 1, true);

                // finally remove castling rights
                self.game_state.king_castle[moving_color] = false;
                self.game_state.queen_castle[moving_color] = false;
            }
        }

        // castling updates for active-color kingside
        if self.game_state.king_castle[moving_color] {
            // any king move will remove castling availability
            if m.piece == Piece::King {
                self.game_state.king_castle[moving_color] = false;
            }

            // rook move on kingside will only remove kingside castling rights
            if m.piece == Piece::Rook && INITIAL_KINGSIDE_ROOK_SQUARES[moving_color] == m.from {
                self.game_state.king_castle[moving_color] = false;
            }
        }

        // castling updates for active-color queenside
        if self.game_state.queen_castle[moving_color] {
            // any king move will remove castling availability
            if m.piece == Piece::King {
                self.game_state.queen_castle[moving_color] = false;
            }

            // rook move on queenside will only remove queenside castling rights
            if m.piece == Piece::Rook && INITIAL_QUEENSIDE_ROOK_SQUARES[moving_color] == m.from {
                self.game_state.queen_castle[moving_color] = false;
            }
        }

        // update the rest of game state
        self.game_state = GameState {
            current_turn: moving_color.opposite(),
            king_castle: self.game_state.king_castle, // update castling rights outside of here, too messy with logic
            queen_castle: self.game_state.queen_castle,
            en_passant_square: match m.flag {
                PawnDoubleMove(square) => Some(square),
                _ => None,
            },
            halfmove: match (m.piece, m.flag) {
                // reset halfmove if pawn push or capture occurred, else increment it
                // other cases for resetting (such as capture promotions) are still pawn moves, so this should match them all
                (Piece::Pawn, _) => 0,
                (_, MoveFlag::Capture(_)) => 0,
                _ => self.game_state.halfmove + 1,
            },
            fullmove: match moving_color {
                // increment fullmove if moving color is black
                Color::Black => self.game_state.fullmove + 1,
                Color::White => self.game_state.fullmove,
            },
        };
    }

    /// Un-makes the last move, restoring the proper board state
    pub fn unmake_move(&mut self) {
        use MoveFlag::*;

        // pop previous move from history
        let (m, prev_state) = match self.history.pop() {
            Some(history) => history,
            None => return, // if no history, return early
        };

        // get color of the side that made the move
        let moving_color = self.game_state.current_turn.opposite();

        // un-make the move, just set move bits m.to -> m.from
        self.colors[moving_color].set_bit_at(m.to, false);
        self.colors[moving_color].set_bit_at(m.from, true);
        self.pieces[m.piece].set_bit_at(m.to, false);
        self.pieces[m.piece].set_bit_at(m.from, true);

        // handle unique move flag cases (castling updated elsewhere)
        match m.flag {
            // nothing more to do
            Quiet => (),

            // need to revert the promoted piece back to a pawn
            Promotion(promoted_piece) => {
                self.pieces[promoted_piece].set_bit_at(m.to, false);
            }

            // need to return the opposing color's piece
            Capture(captured_piece) => {
                self.colors[moving_color.opposite()].set_bit_at(m.to, true);
                self.pieces[captured_piece].set_bit_at(m.to, true);
            }

            // combination of capture and promotion
            CapturePromotion(captured_piece, promoted_piece) => {
                // do promotion changes
                self.pieces[promoted_piece].set_bit_at(m.to, false);

                // do capture changes
                self.colors[moving_color.opposite()].set_bit_at(m.to, true);
                self.pieces[captured_piece].set_bit_at(m.to, true);
            }

            // nothing more to do
            PawnDoubleMove(_) => (),

            // replace enemy pawn at stored square
            EnPassantCapture(enemy_pawn_square) => {
                self.colors[moving_color.opposite()].set_bit_at(enemy_pawn_square, true);
                self.pieces[Piece::Pawn].set_bit_at(enemy_pawn_square, true);
            }

            // reset the rook to the initial square
            KingCastle => {
                // move rook (calculated from m.to)
                self.colors[moving_color].set_bit_at(m.to + 1, true);
                self.colors[moving_color].set_bit_at(m.to - 1, false);

                self.pieces[Piece::Rook].set_bit_at(m.to + 1, true);
                self.pieces[Piece::Rook].set_bit_at(m.to - 1, false);
            }

            // reset the rook to the initial square
            QueenCastle => {
                // move rook (calculated from m.to)
                self.colors[moving_color].set_bit_at(m.to - 2, true);
                self.colors[moving_color].set_bit_at(m.to + 1, false);

                self.pieces[Piece::Rook].set_bit_at(m.to - 2, true);
                self.pieces[Piece::Rook].set_bit_at(m.to + 1, false);
            }
        }

        self.game_state = prev_state;
    }

    /// Returns a structure used for external functions that contains needed info about the board
    pub fn position(&self) -> Position {
        Position {
            active_color: self.game_state.current_turn,

            active_pieces: self.colors[self.game_state.current_turn],
            inactive_pieces: self.colors[self.game_state.current_turn.opposite()],

            en_passant: match self.game_state.en_passant_square {
                Some(square) => Bitboard::shifted_board(square),
                None => Bitboard::EMPTY,
            },

            king_castle_rights: self.game_state.king_castle[self.game_state.current_turn],
            queen_castle_rights: self.game_state.queen_castle[self.game_state.current_turn],

            piece_list: self.get_piece_list(),
        }
    }

    /// Generates a bitboard of pieces matching the given type that can move this turn
    pub fn active_piece_board(&self, piece: Piece) -> Bitboard {
        self.pieces[piece] & self.colors[self.game_state.current_turn]
    }

    /// Generates a bitboard of pieces matching the given type that cannot move this turn
    pub fn inactive_piece_board(&self, piece: Piece) -> Bitboard {
        self.pieces[piece] & self.colors[self.game_state.current_turn.opposite()]
    }

    /// Generates a piece list, containing (if there exists) the piece at every square
    ///
    /// Useful when we have an index of a square and want to know the piece it exists at
    fn get_piece_list(&self) -> [Option<Piece>; BOARD_SIZE] {
        let mut list = [None; BOARD_SIZE];

        for piece in ALL_PIECES {
            for square in self.pieces[piece] {
                list[square] = Some(piece);
            }
        }

        list
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Color::*;
        use Piece::*;

        let mut chars = ['.'; BOARD_SIZE];

        // generate array of characters representing pieces
        for piece in ALL_PIECES {
            for square in self.pieces[piece] {
                let position = Bitboard::shifted_board(square);

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
                if (position & self.colors[White]).is_empty() {
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
            "Castling availability: {} {} {} {} | En passant square: {}\n",
            if self.game_state.king_castle[Color::White] {
                "K"
            } else {
                "-"
            },
            if self.game_state.queen_castle[Color::White] {
                "Q"
            } else {
                "-"
            },
            if self.game_state.king_castle[Color::Black] {
                "k"
            } else {
                "-"
            },
            if self.game_state.queen_castle[Color::Black] {
                "q"
            } else {
                "-"
            },
            match self.game_state.en_passant_square {
                Some(square) => ALGEBRAIC_NOTATION[square].to_string(),
                None => "-".to_string(),
            }
        );
        output.push_str(&castle_info);

        // and write to the formatter
        write!(f, "{}", output)
    }
}
