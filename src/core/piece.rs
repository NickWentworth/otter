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
}

impl From<char> for Piece {
    /// Converts an ascii character to a piece
    fn from(value: char) -> Self {
        use Piece::*;

        match value.to_ascii_uppercase() {
            'P' => Pawn,
            'N' => Knight,
            'B' => Bishop,
            'R' => Rook,
            'Q' => Queen,
            'K' => King,
            _ => panic!("{} is not a valid piece letter!", value),
        }
    }
}

impl From<Piece> for char {
    /// Converts a piece to an uppercase ascii character
    fn from(value: Piece) -> Self {
        use Piece::*;

        match value {
            Pawn => 'P',
            Knight => 'N',
            Bishop => 'B',
            Rook => 'R',
            Queen => 'Q',
            King => 'K',
        }
    }
}

pub const NUM_PIECES: usize = 6;

pub const ALL_PIECES: [Piece; NUM_PIECES] = {
    use Piece::*;
    [Pawn, Knight, Bishop, Rook, Queen, King]
};

pub const PROMOTION_PIECES: [Piece; 4] = {
    use Piece::*;
    [Knight, Bishop, Rook, Queen]
};
