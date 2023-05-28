use crate::types::{Bitboard, Square};

// general utility methods used throughout the program

/// Nicely displays the bitboard, formatted like a chessboard with 0's and 1's.
#[allow(dead_code)]
pub fn display_bitboard(bitboard: Bitboard) {
    for rank in bitboard.to_be_bytes() {
        for i in (0..8).rev() {
            print!("{} ", (rank >> i) & 1);
        }
        println!();
    }
}

/// Right-shifts the bitboard by the specified `amount` (if positive)
///
/// If `amount` is negative, a left-shift is applied with the same magnitude
pub fn conditional_shift_right(board: Bitboard, amount: isize) -> Bitboard {
    if amount >= 0 {
        board >> amount
    } else {
        board << -amount
    }
}

/// Removes the bitboard's most significant 1 bit (in-place) and returns the index that bit was on as a `Square`
pub fn pop_msb_1(bitboard: &mut Bitboard) -> Square {
    let num_zeros = bitboard.leading_zeros() as Square;

    // shift a 1 by number of squares and XOR with bitboard to remove the 1 bit
    *bitboard ^= 0x80_00_00_00_00_00_00_00 >> num_zeros;

    num_zeros
}

/// Tries to convert an algebraic notation string (ex: "b4") to a `Square` on the board, returning an option
pub fn square_from_algebraic(algebraic: &String) -> Option<Square> {
    let file: Square = match algebraic.chars().nth(0)? {
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

    let rank: Square = match algebraic.chars().nth(1)? {
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

/// Used in move generation for bounds checking
///
/// Can be bitwise AND-ed with piece position to mask out pieces on a certain file
pub struct FileBoundMask;
impl FileBoundMask {
    pub const A: Bitboard = 0x7F_7F_7F_7F_7F_7F_7F_7F;
    pub const B: Bitboard = 0xBF_BF_BF_BF_BF_BF_BF_BF;
    // ...
    pub const G: Bitboard = 0xFD_FD_FD_FD_FD_FD_FD_FD;
    pub const H: Bitboard = 0xFE_FE_FE_FE_FE_FE_FE_FE;
}

/// Used in move generation to check if a piece is on a rank
///
/// Can be bitwise AND-ed with piece position to mask out pieces NOT on a certain rank
pub struct RankPositionMask;
impl RankPositionMask {
    // ...
    pub const THIRD: Bitboard = 0x00_00_00_00_00_FF_00_00;
    // ...
    pub const SIXTH: Bitboard = 0x00_00_FF_00_00_00_00_00;
    // ...
}

/// A special bitboard used for indexing, the MSB is set to 1 and all other bits are 0
///
/// To index other squares, this can be right shifted
pub const MSB_BOARD: Bitboard = 0x80_00_00_00_00_00_00_00;
