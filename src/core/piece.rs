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
    pub fn material_value(self) -> f32 {
        use Piece::*;

        match self {
            Pawn => 1.0,
            Knight => 3.0,
            Bishop => 3.0,
            Rook => 5.0,
            Queen => 9.0,
            King => 0.0, // both sides always have a king, so its value isn't needed
        }
    }
}

pub const NUM_PIECES: usize = 6;

use Piece::*;
pub const ALL_PIECES: [Piece; NUM_PIECES] = [Pawn, Knight, Bishop, Rook, Queen, King];
pub const PROMOTION_PIECES: [Piece; 4] = [Knight, Bishop, Rook, Queen];
