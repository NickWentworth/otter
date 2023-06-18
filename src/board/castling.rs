use crate::{
    core::{Color, Piece, Square},
    move_generator::{Move, MoveFlag},
};
use std::fmt::Display;

#[derive(Clone, Copy)]
pub enum CastleSide {
    Kingside,
    Queenside,
}

#[derive(Clone, Copy)]
pub struct CastleRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl CastleRights {
    // initial squares for colors and sides
    const INITIAL_WHITE_KINGSIDE_ROOK: Square = 63;
    const INITIAL_WHITE_QUEENSIDE_ROOK: Square = 56;
    const INITIAL_BLACK_KINGSIDE_ROOK: Square = 7;
    const INITIAL_BLACK_QUEENSIDE_ROOK: Square = 0;

    /// Builds castling rights structure based on FEN string (ex: "KQkq" or "-" for none)
    pub fn from_fen_segment(segment: String) -> CastleRights {
        CastleRights {
            white_kingside: segment.contains('K'),
            white_queenside: segment.contains('Q'),
            black_kingside: segment.contains('k'),
            black_queenside: segment.contains('q'),
        }
    }

    /// Converts structure back to FEN segment
    pub fn to_fen_segment(&self) -> String {
        self.to_string()
    }

    /// Given a `Color` and `CastleSide`, returns castling rights
    pub fn get(&self, color: Color, side: CastleSide) -> bool {
        use CastleSide::*;
        use Color::*;

        match (color, side) {
            (White, Kingside) => self.white_kingside,
            (White, Queenside) => self.white_queenside,
            (Black, Kingside) => self.black_kingside,
            (Black, Queenside) => self.black_queenside,
        }
    }

    /// Given a `Color` and `CastleSide`, sets castling rights
    ///
    /// Generally only used for internal castling management
    fn set(&mut self, color: Color, side: CastleSide, rights: bool) {
        use CastleSide::*;
        use Color::*;

        match (color, side) {
            (White, Kingside) => self.white_kingside = rights,
            (White, Queenside) => self.white_queenside = rights,
            (Black, Kingside) => self.black_kingside = rights,
            (Black, Queenside) => self.black_queenside = rights,
        };
    }

    /// Returns the correct initial rook square index for a given `Color` and `CastleSide`
    fn initial_rook_square(color: Color, side: CastleSide) -> Square {
        use CastleSide::*;
        use Color::*;

        match (color, side) {
            (White, Kingside) => Self::INITIAL_WHITE_KINGSIDE_ROOK,
            (White, Queenside) => Self::INITIAL_WHITE_QUEENSIDE_ROOK,
            (Black, Kingside) => Self::INITIAL_BLACK_KINGSIDE_ROOK,
            (Black, Queenside) => Self::INITIAL_BLACK_QUEENSIDE_ROOK,
        }
    }

    /// Updates the current castling rights based on a move and color making that move
    pub fn update_from_move(&mut self, mov: Move, moving_color: Color) {
        use CastleSide::*;
        use MoveFlag::*;
        use Piece::*;

        // check for changes in moving color's castle rights
        let active_kingside = self.get(moving_color, Kingside);
        let active_queenside = self.get(moving_color, Queenside);

        // if any king move is made for the active side, remove rights
        if active_kingside || active_queenside {
            if mov.piece == King {
                self.set(moving_color, Kingside, false);
                self.set(moving_color, Queenside, false);
            }
        }

        // if any move for active side from initial rook position is made, remove that side's rights
        // don't need to check if a rook made the move, because if the rook has been taken/moved, castle rights are already gone
        if active_kingside && mov.from == Self::initial_rook_square(moving_color, Kingside) {
            self.set(moving_color, Kingside, false);
        }
        if active_queenside && mov.from == Self::initial_rook_square(moving_color, Queenside) {
            self.set(moving_color, Queenside, false);
        }

        // check for changes in non-moving color's castle rights
        let inactive_kingside = self.get(moving_color.opposite(), Kingside);
        let inactive_queenside = self.get(moving_color.opposite(), Queenside);

        // check for capture of opposing piece on initial rook squares
        match mov.flag {
            // only possible source of captures on rook squares
            Capture(_) | CapturePromotion(_, _) => {
                if inactive_kingside
                    && mov.to == Self::initial_rook_square(moving_color.opposite(), Kingside)
                {
                    self.set(moving_color.opposite(), Kingside, false);
                }

                if inactive_queenside
                    && mov.to == Self::initial_rook_square(moving_color.opposite(), Queenside)
                {
                    self.set(moving_color.opposite(), Queenside, false);
                }
            }

            // other flags will not alter other side's castling ability
            _ => (),
        }
    }
}

impl Display for CastleRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        // push matching characters to output string
        if self.white_kingside {
            s.push('K');
        }
        if self.white_queenside {
            s.push('Q');
        }
        if self.black_kingside {
            s.push('k');
        }
        if self.black_queenside {
            s.push('q');
        }

        // if nothing has been pushed, set output to "-"
        if s.is_empty() {
            s = "-".to_string();
        }

        write!(f, "{}", s)
    }
}
