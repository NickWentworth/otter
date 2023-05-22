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

// algebraic notation is in the form "fileRank", ex: a1, b3, ...
pub fn algebraic_to_square(algebriac: &str) -> Option<u8> {
    let file: u8 = match algebriac.chars().nth(0)? {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => return None,
    };

    let rank: u8 = match algebriac.chars().nth(1)? {
        '8' => 0,
        '7' => 1,
        '6' => 2,
        '5' => 3,
        '4' => 4,
        '3' => 5,
        '2' => 6,
        '1' => 7,
        _ => return None,
    };

    Some((rank * 8) + file)
}

// debug method to print the contents of the bitboard
pub fn print_bitboard(b: Bitboard) {
    for rank in b.to_be_bytes() {
        for i in (0..8).rev() {
            print!("{} ", (rank >> i) & 1);
        }
        println!();
    }
}

// used in move generation for bounds checking, can be bitwise AND-ed with piece position to mask out pieces on a certain file
#[repr(u64)]
pub enum FileBoundMask {
    A = 0x7F_7F_7F_7F_7F_7F_7F_7F,
    B = 0xBF_BF_BF_BF_BF_BF_BF_BF,
    // ...
    G = 0xFD_FD_FD_FD_FD_FD_FD_FD,
    H = 0xFE_FE_FE_FE_FE_FE_FE_FE,
}

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

pub enum Color {
    White,
    Black,
}
pub const NUM_COLORS: usize = 2;
index_traits!(Color);
