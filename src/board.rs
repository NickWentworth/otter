use crate::{
    move_generator::{generate_king_moves, generate_knight_moves},
    types::{Bitboard, Color, Piece, NUM_COLORS, NUM_PIECES},
};

pub struct Board {
    pieces: [Bitboard; NUM_PIECES],
    colors: [Bitboard; NUM_COLORS],
}

impl Board {
    pub fn new(fen: &str) -> Self {
        let fen_parts: Vec<&str> = fen.split(' ').collect();

        assert!(fen_parts.len() == 6, "invalid fen string given");

        // build the bitboards for the struct
        let mut pieces: [Bitboard; NUM_PIECES] = [0; NUM_PIECES];
        let mut colors: [Bitboard; NUM_COLORS] = [0; NUM_COLORS];

        // index bitboard to be AND-ed with piece/color bitboards, bitwise looks like 1000...0000
        // instead of being incremented, this is right-shifted to move the 1 over
        let mut index: Bitboard = 0x8000_0000_0000_0000;

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
        // TODO - other game state info from FEN string

        Board { pieces, colors }
    }

    pub fn generate_moves(&self) {
        let king_moves = generate_king_moves(
            // TODO - make method to get pieces of a particular color
            // TODO - use game state to know which side is moving
            self.pieces[Piece::King] & self.colors[Color::White],
            self.colors[Color::White],
        );

        let knight_moves = generate_knight_moves(
            self.pieces[Piece::Knight] & self.colors[Color::White],
            self.colors[Color::White],
        );
    }
}
