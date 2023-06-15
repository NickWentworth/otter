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
}

pub const NUM_PIECES: usize = 6;

use Piece::*;
pub const ALL_PIECES: [Piece; NUM_PIECES] = [Pawn, Knight, Bishop, Rook, Queen, King];
pub const PROMOTION_PIECES: [Piece; 4] = [Knight, Bishop, Rook, Queen];
