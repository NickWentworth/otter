use std::{
    fmt::Display,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Index, IndexMut, Not, Shl,
        ShlAssign, Shr, ShrAssign,
    },
};

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

// -------------------- STRUCTS -------------------- //

#[derive(Clone, Copy, PartialEq)]
pub struct Bitboard(pub u64);

pub type Square = u8;

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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    White,
    Black,
}
pub const NUM_COLORS: usize = 2;

// -------------------- IMPLS -------------------- //

impl Bitboard {
    /// An entirely empty bitboard
    pub const EMPTY: Bitboard = Bitboard(0);

    /// A special bitboard used for indexing, the MSB is set to 1 and all other bits are 0
    pub const MSB: Bitboard = Bitboard(0x80_00_00_00_00_00_00_00);

    /// Returns true if the board has no 1 bits
    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    /// Generates an empty board with a single 1 bit on the given square
    pub fn shifted_board(square: Square) -> Bitboard {
        Self::MSB >> square
    }

    /// Returns `true` if there is a 1 bit at the given square, `false` if not
    pub fn bit_at(self, square: Square) -> bool {
        !(self & Self::shifted_board(square)).is_empty()
    }

    /// Mutates the bitboard by setting the bit at a given square to a 1 (`true`) or 0 (`false`)
    pub fn set_bit_at(&mut self, square: Square, state: bool) {
        if state {
            // set this square to 1 (OR-ing with board of all 0's)
            *self |= Self::shifted_board(square);
        } else {
            // set this square to 0 (AND-ing with board of all 1's)
            *self &= !Self::shifted_board(square);
        }
    }

    /// Returns the square of the first 1 bit in the board, starting from MSB
    pub fn get_first_square(self) -> Square {
        self.0.leading_zeros() as Square
    }

    /// Returns the square of the first 1 bit in the board, starting from LSB
    pub fn get_last_square(self) -> Square {
        (63 - self.0.trailing_zeros()) as Square
    }

    /// Pops the first square from the board (in-place) and returns the square it was popped from
    pub fn pop_first_square(&mut self) -> Square {
        let square = self.get_first_square();
        *self ^= Self::shifted_board(square);
        square
    }
}

impl Display for Bitboard {
    /// Nicely displays the bitboard, formatted like a chessboard with 0's and 1's.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        // build string by checking if each bit is a 0 or 1
        for square in 0..=63 {
            // check if there is a bit on this square
            if (*self & Self::shifted_board(square)).is_empty() {
                s.push('.');
            } else {
                s.push('1');
            }
            s.push(' ');

            // if the next square is the end of line, add newline too
            if (square + 1) % 8 == 0 {
                s.push('\n');
            }
        }

        write!(f, "{}", s)
    }
}

// shifts (for use in directions, negative isizes are treated as shifts in opposite direction)
impl Shr<isize> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: isize) -> Self::Output {
        if rhs >= 0 {
            Bitboard(self.0 >> rhs)
        } else {
            Bitboard(self.0 << -rhs)
        }
    }
}
impl ShrAssign<isize> for Bitboard {
    fn shr_assign(&mut self, rhs: isize) {
        *self = *self >> rhs;
    }
}
impl Shl<isize> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: isize) -> Self::Output {
        if rhs >= 0 {
            Bitboard(self.0 << rhs)
        } else {
            Bitboard(self.0 >> -rhs)
        }
    }
}
impl ShlAssign<isize> for Bitboard {
    fn shl_assign(&mut self, rhs: isize) {
        *self = *self << rhs
    }
}

impl Shr<Square> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: Square) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}
impl ShrAssign<Square> for Bitboard {
    fn shr_assign(&mut self, rhs: Square) {
        *self = *self >> rhs;
    }
}
impl Shl<Square> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: Square) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}
impl ShlAssign<Square> for Bitboard {
    fn shl_assign(&mut self, rhs: Square) {
        *self = *self << rhs
    }
}

// bitwise operators (same as general u64 operations)
impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}
impl BitAnd for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}
impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
impl BitOr for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}
impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
impl BitXor for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}
impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl Color {
    /// Returns the opposite color to the given one
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

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
