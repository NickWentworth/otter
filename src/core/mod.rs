use std::ops::{Index, IndexMut};

mod bitboard;
mod color;
mod piece;
mod square;

pub use bitboard::*;
pub use color::*;
pub use piece::*;
pub use square::*;

/// Adds `Index` and `IndexMut` traits so that arrays can be indexed by enums
macro_rules! index_traits {
    ( $index_type:ty, $array_type:ty ) => {
        impl Index<$index_type> for [$array_type] {
            type Output = $array_type;

            fn index(&self, index: $index_type) -> &Self::Output {
                &self[index as usize]
            }
        }

        impl IndexMut<$index_type> for [$array_type] {
            fn index_mut(&mut self, index: $index_type) -> &mut Self::Output {
                &mut self[index as usize]
            }
        }
    };
}

// pieces and colors arrays within Board struct
index_traits!(Piece, Bitboard);
index_traits!(Color, Bitboard);

// castling rights within Board struct
index_traits!(Color, bool);

// initial rook squares for castling updates in Board struct
index_traits!(Color, Square);

// bitboard lookup tables
index_traits!(Color, [Bitboard; BOARD_SIZE]);
