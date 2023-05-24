use std::ops::{Index, IndexMut};

/// Adds `Index` and `IndexMut` traits so that bitboard arrays can be indexed by enums
macro_rules! index_traits {
    ( $t:ty ) => {
        impl Index<$t> for [Bitboard] {
            type Output = Bitboard;

            fn index(&self, index: $t) -> &Self::Output {
                &self[index as usize]
            }
        }

        impl IndexMut<$t> for [Bitboard] {
            fn index_mut(&mut self, index: $t) -> &mut Self::Output {
                &mut self[index as usize]
            }
        }
    };
}

// TODO - seems confusing, may be better if things are done in reverse (63=MSB, 0=LSB)
/*
Bitboard bits to Square values:
(MSB) 0  1  2  3  4  5  6  7
      8  9  10 11 12 13 14 15
      16 17 18 19 20 21 22 23
      24 25 26 27 28 29 30 31
      32 33 34 35 36 37 38 39
      40 41 42 43 44 45 46 47
      48 49 50 51 52 53 54 55
      56 57 58 59 60 61 62 63 (LSB)
*/
pub type Bitboard = u64;
pub type Square = u8;

#[derive(Clone, Copy)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
pub const NUM_PIECES: usize = 6;
index_traits!(Piece);

#[derive(Clone, Copy)]
pub enum Color {
    White,
    Black,
}
pub const NUM_COLORS: usize = 2;
index_traits!(Color);

impl Color {
    /// Returns the opposite color to the given one
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
