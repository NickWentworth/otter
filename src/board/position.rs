use crate::core::{Bitboard, Color, Piece, BOARD_SIZE};

use super::castling::{CastleRights, CastleSide};

/// Contains needed info to represent the state of the board, to be used by external classes
pub struct Position {
    pub active_color: Color,

    pub active_pieces: Bitboard,
    pub inactive_pieces: Bitboard,

    pub en_passant: Bitboard,
    pub castle_rights: CastleRights,

    pub piece_list: [Option<Piece>; BOARD_SIZE],
}

impl Position {
    /// Returns a `Bitboard` containing all pieces, regardless of color
    pub fn all_pieces(&self) -> Bitboard {
        self.active_pieces | self.inactive_pieces
    }

    /// Returns kingside rights of active side
    pub fn active_kingside_rights(&self) -> bool {
        self.castle_rights
            .get(self.active_color, CastleSide::Kingside)
    }

    /// Returns queenside rights of active side
    pub fn active_queenside_rights(&self) -> bool {
        self.castle_rights
            .get(self.active_color, CastleSide::Queenside)
    }
}
