use crate::search::Score;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    /// Returns true if the piece attacks by sliding
    pub fn is_sliding(self) -> bool {
        use Piece::*;

        match self {
            Bishop | Rook | Queen => true,
            Pawn | Knight | King => false,
        }
    }

    /// Converts a piece to a relative material value
    pub fn material_value(self) -> Score {
        use Piece::*;

        match self {
            Pawn => 100,
            Knight => 300,
            Bishop => 300,
            Rook => 500,
            Queen => 900,
            King => 0, // both sides always have a king, so its value isn't needed
        }
    }

    /// Converts a piece to its commonly used symbol
    pub fn symbol(self) -> char {
        use Piece::*;

        match self {
            Pawn => 'p',
            Knight => 'n',
            Bishop => 'b',
            Rook => 'r',
            Queen => 'q',
            King => 'k',
        }
    }
}

pub const NUM_PIECES: usize = 6;

use Piece::*;
pub const ALL_PIECES: [Piece; NUM_PIECES] = [Pawn, Knight, Bishop, Rook, Queen, King];
pub const PROMOTION_PIECES: [Piece; 4] = [Knight, Bishop, Rook, Queen];
