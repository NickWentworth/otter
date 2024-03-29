use crate::core::{
    Bitboard, Color, Piece, Square, ALGEBRAIC_NOTATION, ALL_PIECES, BOARD_SIZE, NUM_COLORS,
    NUM_PIECES, PROMOTION_PIECES,
};
use std::fmt::Display;

mod castling;
mod fen;
mod move_generator;
mod perft;
mod zobrist;

pub use move_generator::{Magic, Move, MoveFlag};
pub use perft::perft_divide;
pub use zobrist::ZobristHash;

use castling::{CastleRights, CastleSide};
use fen::{check_valid_fen, DEFAULT_FEN};
use move_generator::MoveGenerator;
use zobrist::ZOBRIST;

/// Variables related to conditions of the game
#[derive(Clone, Copy)]
struct GameState {
    current_turn: Color,
    castle_rights: CastleRights,
    en_passant_square: Option<Square>,
    halfmove: u32, // halfmove counter, incremented after each color's move
    fullmove: u32, // fullmove counter, only incremented after black's move
}

/// Overall representation of a chess game
pub struct Board {
    pieces: [Bitboard; NUM_PIECES],
    colors: [Bitboard; NUM_COLORS],
    game_state: GameState,

    // alternate piece location representation allowing indexing squares to find the piece on that square
    piece_list: [Option<Piece>; BOARD_SIZE],

    // stack containing moves and matching info needed to unmake the previously made move
    move_history: Vec<(Move, GameState)>,

    // stack containing previous hashes used for detection of threefold repetition
    position_history: Vec<ZobristHash>,
}

impl Board {
    /// Generates a new `Board` from a given FEN string
    ///
    /// The FEN string is validated, if invalid the board is set to the start state of the chess game
    pub fn new(fen: &str) -> Board {
        // check if the given string is valid
        let fen_parts = if check_valid_fen(fen) {
            fen
        } else {
            // if not, just use the default fen string
            println!("Invalid FEN! Reverting to starting position.");
            DEFAULT_FEN
        }
        .split(" ")
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

        // build the bitboards for the struct
        let mut pieces = [Bitboard::EMPTY; NUM_PIECES];
        let mut colors = [Bitboard::EMPTY; NUM_COLORS];

        // index bitboard to be AND-ed with piece/color bitboards, bitwise looks like 1000...0000
        // instead of being incremented, this is right-shifted to move the 1 over
        let mut index = Bitboard::MSB;

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
                    // change bitboards based on casing and character
                    colors[Color::from(c)] |= index;
                    pieces[Piece::from(c)] |= index;

                    // finally increment index
                    index >>= 1 as Square;
                }
            }
        }

        let mut b = Board {
            pieces,
            colors,
            game_state: GameState {
                current_turn: if fen_parts[1] == "w" {
                    Color::White
                } else {
                    Color::Black
                },
                castle_rights: CastleRights::from_fen_segment(fen_parts[2].clone()),
                en_passant_square: ALGEBRAIC_NOTATION.iter().position(|&s| s == fen_parts[3]),
                halfmove: fen_parts[4].parse().unwrap(),
                fullmove: fen_parts[5].parse().unwrap(),
            },
            piece_list: [None; BOARD_SIZE],
            move_history: Vec::new(),
            position_history: Vec::new(),
        };

        b.piece_list = b.build_piece_list();

        // other systems expect board to be in a valid state, so check if it is valid
        if !b.is_legal_position() {
            println!("Invalid FEN! Reverting to starting position.");
            b = Board::default();
        }

        b
    }

    /// Returns a FEN string representing the current board position
    pub fn to_fen(&self) -> String {
        use Color::*;

        let mut fen = String::new();

        for (square, piece) in self.piece_list.iter().enumerate() {
            // get the symbol of this square
            let symbol = match piece {
                None => '1',
                Some(p) => char::from(*p),
            };

            // add proper casing if needed
            if self.colors[White].bit_at(square) {
                fen.push(White.to_char(symbol));
            } else if self.colors[Black].bit_at(square) {
                fen.push(Black.to_char(symbol));
            } else {
                // if not in either colors bitboard, just push the regular symbol
                fen.push(symbol);
            }

            // if we are moving to the next rank, then add a slash
            if (square + 1) % 8 == 0 {
                fen.push('/');
            }
        }

        // replace repeating ones
        fen = fen.replace("11111111", "8");
        fen = fen.replace("1111111", "7");
        fen = fen.replace("111111", "6");
        fen = fen.replace("11111", "5");
        fen = fen.replace("1111", "4");
        fen = fen.replace("111", "3");
        fen = fen.replace("11", "2");

        // after this point, we have an extra slash at the end, so remove it for a space
        fen.remove(fen.chars().count() - 1);
        fen.push(' ');

        // active color
        fen.push(match self.active_color() {
            White => 'w',
            Black => 'b',
        });
        fen.push(' ');

        // castling data
        fen.push_str(&self.game_state.castle_rights.to_fen_segment());
        fen.push(' ');

        // en passant target square
        fen.push_str(match self.game_state.en_passant_square {
            Some(square) => ALGEBRAIC_NOTATION[square],
            None => "-",
        });
        fen.push(' ');

        // halfmove
        fen.push_str(&self.game_state.halfmove.to_string());
        fen.push(' ');

        // fullmove
        fen.push_str(&self.game_state.fullmove.to_string());

        fen
    }

    /// Makes the given move and updates game state accordingly
    ///
    /// Assumes `m` is a valid and legal move
    pub fn make_move(&mut self, m: Move) {
        // push move and current game state to stack
        self.move_history.push((m, self.game_state));
        self.position_history.push(self.zobrist());

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
            }

            // move the rook to the correct square and change castling rights
            QueenCastle => {
                // move rook (calculated from m.to)
                self.colors[moving_color].set_bit_at(m.to - 2, false);
                self.colors[moving_color].set_bit_at(m.to + 1, true);

                self.pieces[Piece::Rook].set_bit_at(m.to - 2, false);
                self.pieces[Piece::Rook].set_bit_at(m.to + 1, true);
            }
        }

        // update the game state
        self.game_state.current_turn = moving_color.opposite();

        self.game_state.en_passant_square = match m.flag {
            PawnDoubleMove(square) => Some(square),
            _ => None,
        };

        self.game_state
            .castle_rights
            .update_from_move(m, moving_color);

        self.game_state.halfmove = match (m.piece, m.flag) {
            // reset halfmove if pawn push or capture occurred, else increment it
            // other cases for resetting (such as capture promotions) are still pawn moves, so this should match them all
            (Piece::Pawn, _) => 0,
            (_, MoveFlag::Capture(_)) => 0,
            _ => self.game_state.halfmove + 1,
        };

        if moving_color == Color::Black {
            self.game_state.fullmove += 1;
        }

        // refresh the piece list
        self.piece_list = self.build_piece_list();
    }

    /// Un-makes the last move, restoring the proper board state
    pub fn unmake_move(&mut self) {
        use MoveFlag::*;

        // pop previous move from history
        let (m, prev_state) = match self.move_history.pop() {
            Some(history) => history,
            None => return, // if no history, return early
        };

        // also pop this position from zobrist history
        self.position_history.pop();

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

        self.piece_list = self.build_piece_list();
    }

    /// Generates all legal moves from this position
    pub fn generate_moves(&self) -> Vec<Move> {
        MoveGenerator::generate_moves(&self)
    }

    /// Generates all legal capture moves from this position
    // TODO - add capture-only generation to move generator, this filtering is too slow
    pub fn generate_captures(&self) -> Vec<Move> {
        MoveGenerator::generate_moves(&self)
            .into_iter()
            .filter(|mov| mov.is_capture())
            .collect()
    }

    /// Returns whether or not the active color is in check in this position
    pub fn in_check(&self) -> bool {
        MoveGenerator::in_check(&self)
    }

    /// Checks for cases where a draw is possible and returns whether or not it is
    pub fn is_drawable(&self) -> bool {
        // check for 50 halfmove rule
        if self.game_state.halfmove >= 50 {
            return true;
        }

        // check for threefold repetitions
        // only the current position is checked for repetitions, so ensure that after each move this is checked
        let current_hash = self.zobrist();
        let mut matches = 0;

        for hash in self.position_history.iter() {
            if *hash == current_hash {
                matches += 1;

                if matches >= 3 {
                    return true;
                }
            }
        }

        false
    }

    /// Returns the piece type at the given square or `None` if no piece is at the square
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.piece_list[square]
    }

    /// Returns the current moving color
    pub fn active_color(&self) -> Color {
        self.game_state.current_turn
    }

    /// Returns the current non-moving color
    pub fn inactive_color(&self) -> Color {
        self.game_state.current_turn.opposite()
    }

    /// Returns a bitboard with bits wherever there are active pieces
    pub fn active_pieces(&self) -> Bitboard {
        self.colors[self.game_state.current_turn]
    }

    /// Returns a bitboard with bits wherever there are inactive pieces
    pub fn inactive_pieces(&self) -> Bitboard {
        self.colors[self.game_state.current_turn.opposite()]
    }

    /// Returns a bitboard with bits wherever there are any pieces, either color
    pub fn all_pieces(&self) -> Bitboard {
        self.colors[Color::White] | self.colors[Color::Black]
    }

    /// Returns a board with a bit at the en passant capture square, if it exists
    pub fn en_passant_board(&self) -> Bitboard {
        match self.game_state.en_passant_square {
            Some(square) => Bitboard::shifted_board(square),
            None => Bitboard::EMPTY,
        }
    }

    /// Returns kingside rights of active side
    pub fn active_kingside_rights(&self) -> bool {
        self.game_state
            .castle_rights
            .get(self.game_state.current_turn, CastleSide::Kingside)
    }

    /// Returns queenside rights of active side
    pub fn active_queenside_rights(&self) -> bool {
        self.game_state
            .castle_rights
            .get(self.game_state.current_turn, CastleSide::Queenside)
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
    // TODO - incrementally update this list instead of generating it fresh every time
    fn build_piece_list(&self) -> [Option<Piece>; BOARD_SIZE] {
        let mut list = [None; BOARD_SIZE];

        for piece in ALL_PIECES {
            for square in self.pieces[piece] {
                list[square] = Some(piece);
            }
        }

        list
    }

    /// Generates a zobrist hash value representing the current board state
    // TODO - incrementally update this hash instead of generating it fresh every time
    pub fn zobrist(&self) -> ZobristHash {
        use CastleSide::*;
        use Color::*;

        let mut hash = 0;

        // squares
        for (square, piece_option) in self.piece_list.iter().enumerate() {
            match piece_option {
                Some(piece) => {
                    let color = match self.colors[Color::White].bit_at(square) {
                        true => Color::White,
                        false => Color::Black,
                    };

                    hash ^= ZOBRIST.piece(square, *piece, color);
                }
                None => (),
            }
        }

        // castling
        for color in [White, Black] {
            for castle_side in [Kingside, Queenside] {
                if self.game_state.castle_rights.get(color, castle_side) {
                    hash ^= ZOBRIST.castling(castle_side, color);
                }
            }
        }

        // active turn
        hash ^= ZOBRIST.active(self.game_state.current_turn);

        // en passant
        hash ^= ZOBRIST.en_passant(self.game_state.en_passant_square);

        hash
    }

    /// Checks if the current board position is in a legal state
    fn is_legal_position(&self) -> bool {
        use Color::*;
        use MoveFlag::*;
        use Piece::*;

        // check for correct piece counts per color
        for color in [White, Black] {
            let mut pieces = [0; NUM_PIECES];

            // count up pieces
            for square in self.colors[color] {
                // should be a piece at this square
                let piece = self.piece_list[square].unwrap();
                pieces[piece] += 1;
            }

            // now check that counts are valid
            // correct amount of un-promotable pieces
            if pieces[King] != King.initial_count() {
                return false;
            }
            if pieces[Pawn] > Pawn.initial_count() {
                return false;
            }

            // possible to have promoted pawns to get to this position
            let mut missing_pawns = Pawn.initial_count() - pieces[Pawn];
            for promotable in PROMOTION_PIECES {
                // if there are less than or equal to the initial count of this piece,
                // then it isn't guaranteed that a pawn was promoted
                if pieces[promotable] <= promotable.initial_count() {
                    continue;
                }

                // get the number of pieces that must have been promoted for this piece type
                let promoted_pieces = pieces[promotable] - promotable.initial_count();

                // not enough missing pawns to have promoted this many pieces
                if promoted_pieces > missing_pawns {
                    return false;
                }

                // else, subtract the promoted pieces from missing pawns
                missing_pawns -= promoted_pieces;
            }
        }

        // the current moving side cannot have the opposing king in check
        for capture in self.generate_captures() {
            // ensure a king is not being captured right now
            match capture.flag {
                Capture(King) | CapturePromotion(King, _) => return false,
                _ => (),
            }
        }

        true
    }
}

impl Default for Board {
    /// Setup a board from the starting position
    fn default() -> Self {
        Board::new(DEFAULT_FEN)
    }
}

impl Clone for Board {
    /// Creates a shallow copy of the board, meaning move history is not stored and only moves from this point on can be undone
    fn clone(&self) -> Self {
        Self {
            pieces: self.pieces,
            colors: self.colors,
            game_state: self.game_state,
            piece_list: self.piece_list,
            move_history: Vec::new(),
            position_history: Vec::new(),
        }
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
            "Castling availability: {} | En passant square: {}\n",
            self.game_state.castle_rights.to_fen_segment(),
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

#[cfg(test)]
mod tests {
    use super::*;
    
    const TEST_FENS: [&str; 3] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    ];

    #[test]
    fn test_board_to_fen() {
        for fen in TEST_FENS {
            let b = Board::new(fen);
            assert_eq!(b.to_fen(), fen.to_string());
        }
    }

    #[test]
    fn test_board_zobrist() {
        let default_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        // check initial hash is the same
        let mut b1 = Board::new(default_fen);
        let mut b2 = Board::new(default_fen);

        assert_eq!(b1.zobrist(), b2.zobrist());

        // make the same moves in different orders
        let white_e2e4 = Move { from: 52, to: 36, piece: Piece::Pawn, flag: MoveFlag::Quiet };
        let white_g1f3 = Move { from: 62, to: 45, piece: Piece::Knight, flag: MoveFlag::Quiet };
        
        let black_e7e5 = Move { from: 12, to: 28, piece: Piece::Pawn, flag: MoveFlag::Quiet };
        let black_b8c6 = Move { from: 1, to: 18, piece: Piece::Knight, flag: MoveFlag::Quiet };

        b1.make_move(white_e2e4);
        b1.make_move(black_e7e5);
        b1.make_move(white_g1f3);
        b1.make_move(black_b8c6);

        b2.make_move(white_g1f3);
        b2.make_move(black_b8c6);
        b2.make_move(white_e2e4);
        b2.make_move(black_e7e5);

        // check that same transpositions have the same hashes
        assert_eq!(b1.zobrist(), b2.zobrist());
    }
}

