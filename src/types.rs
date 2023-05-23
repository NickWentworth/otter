use std::ops::{Index, IndexMut};

// adds Index and IndexMut traits so that bitboard arrays can be indexed by enums
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

pub type Bitboard = u64;
pub type Square = u8;

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
