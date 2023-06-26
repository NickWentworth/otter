use rand::Rng;

use crate::core::{Color, Piece, Square, BOARD_SIZE, NUM_COLORS, NUM_PIECES};

use super::castling::{CastleSide, NUM_CASTLE_SIDES};

/// Type for the underlying hash value
pub type ZobristHash = u64;

/// Contains arrays of generated random values, each corresponding to a possible modification of the board state
/// 
/// These values can be XOR-ed with a current hash whenever a piece is moved, castling rights are changed, etc, to have
/// a readily available hash value for the board state.
pub struct ZobristValues {
    // random number for each square, piece, and color
    pieces: [[[ZobristHash; BOARD_SIZE]; NUM_PIECES]; NUM_COLORS],

    // random number for castling rights
    castling: [[ZobristHash; NUM_CASTLE_SIDES]; NUM_COLORS],

    // random number for moving side
    active: [ZobristHash; NUM_COLORS],

    // random value for en passant square
    en_passant: [ZobristHash; BOARD_SIZE + 1],
}

impl ZobristValues {
    pub fn new() -> ZobristValues {
        // reference random number generator
        let mut rng = rand::thread_rng();

        let mut z = ZobristValues {
            pieces: [[[0; BOARD_SIZE]; NUM_PIECES]; NUM_COLORS],
            castling: [[0; NUM_CASTLE_SIDES]; NUM_COLORS],
            active: [0; NUM_COLORS],
            en_passant: [0; BOARD_SIZE + 1],
        };

        for i in z.pieces.iter_mut() {
            for j in i.iter_mut() {
                for k in j.iter_mut() {
                    *k = rng.gen_range(ZobristHash::MIN..ZobristHash::MAX);
                }
            }
        }

        for i in z.castling.iter_mut() {
            for j in i.iter_mut() {
                *j = rng.gen_range(ZobristHash::MIN..ZobristHash::MAX);
            }
        }

        for i in z.active.iter_mut() {
            *i = rng.gen_range(ZobristHash::MIN..ZobristHash::MAX);
        }

        for i in z.en_passant.iter_mut() {
            *i = rng.gen_range(ZobristHash::MIN..ZobristHash::MAX);
        }

        z
    }

    pub fn piece(&self, square: Square, piece: Piece, color: Color) -> ZobristHash {
        self.pieces[color as usize][piece as usize][square]
    }

    pub fn castling(&self, castle_side: CastleSide, color: Color) -> ZobristHash {
        self.castling[color as usize][castle_side as usize]
    }

    pub fn active(&self, color: Color) -> ZobristHash {
        self.active[color as usize]
    }

    pub fn en_passant(&self, en_passant_square: Option<Square>) -> ZobristHash {
        match en_passant_square {
            Some(square) => self.en_passant[square],
            None => self.en_passant[BOARD_SIZE],
        }
    }
}
