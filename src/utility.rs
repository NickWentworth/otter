use crate::{
    bitboard::{Bitboard, Square},
    types::NUM_COLORS,
};

// general utility methods used throughout the program

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
    pub const A: Bitboard = Bitboard(0x7F_7F_7F_7F_7F_7F_7F_7F);
    pub const B: Bitboard = Bitboard(0xBF_BF_BF_BF_BF_BF_BF_BF);
    // ...
    pub const G: Bitboard = Bitboard(0xFD_FD_FD_FD_FD_FD_FD_FD);
    pub const H: Bitboard = Bitboard(0xFE_FE_FE_FE_FE_FE_FE_FE);
}

/// Used in move generation to check if a piece is on a rank
///
/// Can be bitwise AND-ed with piece position to mask out pieces NOT on a certain rank
pub struct RankPositionMask;
impl RankPositionMask {
    // ...
    pub const THIRD: Bitboard = Bitboard(0x00_00_00_00_00_FF_00_00);
    // ...
    pub const SIXTH: Bitboard = Bitboard(0x00_00_FF_00_00_00_00_00);
    // ...
}

/// Can be bitwise AND-ed to test for valid castling squares
///
/// Indexed by the `Color` of the king being castled
pub struct CastleMask;
impl CastleMask {
    pub const KINGSIDE: [Bitboard; NUM_COLORS] = [
        Bitboard(0x00_00_00_00_00_00_00_06),
        Bitboard(0x06_00_00_00_00_00_00_00),
    ];

    pub const QUEENSIDE: [Bitboard; NUM_COLORS] = [
        Bitboard(0x00_00_00_00_00_00_00_70),
        Bitboard(0x70_00_00_00_00_00_00_00),
    ];
}
