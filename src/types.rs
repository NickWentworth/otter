use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, PartialEq)]
pub struct Bitboard(pub u64);

pub type Square = u8;

/// Adds `Index` and `IndexMut` traits so that bitboard arrays can be indexed by enums
macro_rules! index_traits {
    ( $t1:ty, $t2:ty ) => {
        impl Index<$t1> for [$t2] {
            type Output = $t2;

            fn index(&self, index: $t1) -> &Self::Output {
                &self[index as usize]
            }
        }

        impl IndexMut<$t1> for [$t2] {
            fn index_mut(&mut self, index: $t1) -> &mut Self::Output {
                &mut self[index as usize]
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
pub const NUM_PIECES: usize = 6;
index_traits!(Piece, Bitboard);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}
pub const NUM_COLORS: usize = 2;
index_traits!(Color, Bitboard);
index_traits!(Color, bool);

impl Color {
    /// Returns the opposite color to the given one
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
