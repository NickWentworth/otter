use crate::{
    fen::{check_valid_fen, DEFAULT_FEN},
    move_generator::{generate_king_moves, generate_knight_moves},
    types::{Bitboard, Color, Piece, Square, NUM_COLORS, NUM_PIECES},
    utility::square_from_algebraic,
};

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

pub struct Board {
    pieces: [Bitboard; NUM_PIECES],
    colors: [Bitboard; NUM_COLORS],
    game_state: GameState,
}

impl Board {
    pub fn new(fen: &str) -> Self {
        // check if the given string is valid
        let fen_parts: Vec<&str> = if check_valid_fen(fen) {
            fen
        } else {
            // if not, just use the default fen string for now
            DEFAULT_FEN
        }
        .split(' ')
        .collect();

        // build the bitboards for the struct
        let mut pieces: [Bitboard; NUM_PIECES] = [0; NUM_PIECES];
        let mut colors: [Bitboard; NUM_COLORS] = [0; NUM_COLORS];

        // index bitboard to be AND-ed with piece/color bitboards, bitwise looks like 1000...0000
        // instead of being incremented, this is right-shifted to move the 1 over
        let mut index: Bitboard = 0x80_00_00_00_00_00_00_00;

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
                en_passant_square: square_from_algebraic(fen_parts[3]), // will handle correctly if passed "-"
                halfmove: fen_parts[4].parse().unwrap(),
                fullmove: fen_parts[5].parse().unwrap(),
            },
        }
    }

    pub fn generate_moves(&self) {
        let king_moves = generate_king_moves(
            // TODO - make method to get pieces of a particular color
            // TODO - use game state to know which side is moving
            self.pieces[Piece::King] & self.colors[Color::White],
            self.colors[Color::White],
        );

        // TODO - generate move methods should probably return move lists instead of bitboards
        //        for example, if two pieces can move to the same spot, data is lost
        let knight_moves = generate_knight_moves(
            self.pieces[Piece::Knight] & self.colors[Color::White],
            self.colors[Color::White],
        );
    }
}
