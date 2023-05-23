use crate::types::{Bitboard, Square};

// general utility methods used throughout the program

// nicely displays the bitboard to look like the chessboard
#[allow(dead_code)]
pub fn display_bitboard(bitboard: Bitboard) {
    for rank in bitboard.to_be_bytes() {
        for i in (0..8).rev() {
            print!("{} ", (rank >> i) & 1);
        }
        println!();
    }
}

// takes in a bitboard, removes its most significant 1 bit in-place, and returns the index it occurs at as a square
pub fn pop_msb_1(bitboard: &mut Bitboard) -> Square {
    let num_zeros = bitboard.leading_zeros() as Square;

    // shift a 1 by number of squares and XOR with bitboard to remove the 1 bit
    *bitboard ^= 0x80_00_00_00_00_00_00_00 >> num_zeros;

    num_zeros
}

// converts a string in algebraic notation (ex: b4) to integer type
pub fn square_from_algebraic(algebraic: &str) -> Option<Square> {
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

// used in move generation for bounds checking, can be bitwise AND-ed with piece position to mask out pieces on a certain file
pub struct FileBoundMask;
impl FileBoundMask {
    pub const A: Bitboard = 0x7F_7F_7F_7F_7F_7F_7F_7F;
    pub const B: Bitboard = 0xBF_BF_BF_BF_BF_BF_BF_BF;
    // ...
    pub const G: Bitboard = 0xFD_FD_FD_FD_FD_FD_FD_FD;
    pub const H: Bitboard = 0xFE_FE_FE_FE_FE_FE_FE_FE;
}

// used in move generation to check if a piece is on a rank, can be bitwise AND-ed with piece position to mask out pieces NOT on a certain rank
pub struct RankPositionMask;
impl RankPositionMask {
    // ...
    pub const THIRD: Bitboard = 0x00_00_00_00_00_FF_00_00;
    // ...
    pub const SIXTH: Bitboard = 0x00_00_FF_00_00_00_00_00;
    // ...
}
