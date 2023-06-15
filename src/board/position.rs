use crate::core::{Bitboard, Color, Piece, BOARD_SIZE};

/// Contains needed info to represent the state of the board, to be used by external classes
pub struct Position {
    pub active_color: Color,

    pub active_pieces: Bitboard,
    pub inactive_pieces: Bitboard,

    pub en_passant: Bitboard,
    pub king_castle_rights: bool,
    pub queen_castle_rights: bool,

    pub piece_list: [Option<Piece>; BOARD_SIZE],
}

impl Position {
    /// Returns a `Bitboard` containing all pieces, regardless of color
    pub fn all_pieces(&self) -> Bitboard {
        self.active_pieces | self.inactive_pieces
    }
}
