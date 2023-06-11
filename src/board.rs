use crate::{
    move_generator::{Move, MoveFlag},
    types::{Bitboard, Color, Piece, Square, ALL_PIECES, BOARD_SIZE, NUM_COLORS, NUM_PIECES},
};
use std::fmt::Display;

mod fen;

use fen::{check_valid_fen, DEFAULT_FEN};

/// Variables related to conditions of the game
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
}

/// Contains important info for move generation that is fetched once and used multiple times
pub struct MoveGenBoardInfo {
    pub active_color: Color,
    pub inactive_color: Color,

    pub same_pieces: Bitboard,
    pub opposing_pieces: Bitboard,
    pub all_pieces: Bitboard,
    pub no_pieces: Bitboard,

    pub en_passant: Bitboard,
    pub king_castle_rights: bool,
    pub queen_castle_rights: bool,

    pub piece_list: [Option<Piece>; BOARD_SIZE],
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

        // check if rook/king has been moved to change castling rights
        let kingside_rights = self.game_state.king_castle[moving_color];
        let queenside_rights = self.game_state.queen_castle[moving_color];

        // TODO - add these rook squares as constants elsewhere
        // kingside check
        if kingside_rights && (m.piece == Piece::Rook) && (m.from == 7 || m.from == 63) {
            self.game_state.king_castle[moving_color] = false;
        }

        // TODO - add these rook squares as constants elsewhere
        // queenside check
        if queenside_rights && (m.piece == Piece::Rook) && (m.from == 0 || m.from == 56) {
            self.game_state.queen_castle[moving_color] = false;
        }
    }

    /// Returns a structure used for move generation that contains needed info about the board
    pub fn get_board_info(&self) -> MoveGenBoardInfo {
        let active_color = self.game_state.current_turn;
        let inactive_color = active_color.opposite();

        let same_pieces = self.colors[active_color];
        let opposing_pieces = self.colors[inactive_color];
        let all_pieces = same_pieces | opposing_pieces;
        let no_pieces = !all_pieces;

        let en_passant = match self.game_state.en_passant_square {
            Some(square) => Bitboard::shifted_board(square),
            None => Bitboard::EMPTY,
        };

        let king_castle_rights = self.game_state.king_castle[active_color];
        let queen_castle_rights = self.game_state.queen_castle[active_color];

        MoveGenBoardInfo {
            active_color,
            inactive_color,
            same_pieces,
            opposing_pieces,
            all_pieces,
            no_pieces,
            en_passant,
            king_castle_rights,
            queen_castle_rights,
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
                Some(square) => square.to_string(),
                None => "-".to_string(),
            }
        );
        output.push_str(&castle_info);

        // and write to the formatter
        write!(f, "{}", output)
    }
}

/// Tries to convert an algebraic notation string (ex: "b4") to a `Square` on the board, returning an option
fn square_from_algebraic(algebraic: &String) -> Option<Square> {
    let file: Square = match algebraic.chars().nth(0)? {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => return None,
    };

    let rank: Square = match algebraic.chars().nth(1)? {
        '8' => 0,
        '7' => 1,
        '6' => 2,
        '5' => 3,
        '4' => 4,
        '3' => 5,
        '2' => 6,
        '1' => 7,
        _ => return None,
    };

    Some((rank * 8) + file)
}
