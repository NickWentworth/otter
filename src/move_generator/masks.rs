use crate::types::{Bitboard, NUM_COLORS};

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
    // single rank checks
    pub const SECOND: Bitboard = Bitboard(0x00_00_00_00_00_00_FF_00);
    pub const SEVENTH: Bitboard = Bitboard(0x00_FF_00_00_00_00_00_00);

    // check for pawns on promotion squares
    // don't need to separate the promotion squares for each side, only white pawns can move to rank 8 and black to rank 1
    pub const PROMOTION: Bitboard = Bitboard(0xFF_00_00_00_00_00_00_FF);
}

/// Used in move generation to check if castling squares are valid according to rules
///
/// Can be bitwise AND-ed to test if castling squares are under attack and indexed by the `Color` of the castling side
pub struct CastleMask;
impl CastleMask {
    // empty squares that cannot have pieces on them for castling
    pub const KINGSIDE_EMPTY: [Bitboard; NUM_COLORS] = [
        Bitboard(0x00_00_00_00_00_00_00_06), // white
        Bitboard(0x06_00_00_00_00_00_00_00), // black
    ];

    pub const QUEENSIDE_EMPTY: [Bitboard; NUM_COLORS] = [
        Bitboard(0x00_00_00_00_00_00_00_70), // white
        Bitboard(0x70_00_00_00_00_00_00_00), // black
    ];

    // square that cannot be attacked for castling to be valid
    pub const KINGSIDE_SAFE: [Bitboard; NUM_COLORS] = [
        Bitboard(0x00_00_00_00_00_00_00_0E), // white
        Bitboard(0x0E_00_00_00_00_00_00_00), // black
    ];

    pub const QUEENSIDE_SAFE: [Bitboard; NUM_COLORS] = [
        Bitboard(0x00_00_00_00_00_00_00_38), // white
        Bitboard(0x38_00_00_00_00_00_00_00), // black
    ];
}
