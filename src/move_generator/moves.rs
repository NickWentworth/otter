use crate::types::{Piece, Square};

#[derive(Clone, Copy, Debug)]
pub enum MoveFlag {
    Quiet,                          // nothing special, regular move that doesn't have any flags
    Capture(Piece),                 // opponent piece that was captured
    Promotion(Piece),               // pawn was promoted into a piece
    CapturePromotion(Piece, Piece), // opponent piece that was captured as well as the piece promoted into
    PawnDoubleMove(Square),         // pawn double moved and stores the en passant square
    EnPassantCapture(Square),       // holds the square of the captured (just en passant-ed) pawn
    KingCastle,                     // kingside castle
    QueenCastle,                    // queenside castle
}

/// Describes a move on the board and information related to that move
#[derive(Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub flag: MoveFlag,
}

impl Move {
    pub fn new(from: Square, to: Square, piece: Piece) -> Move {
        Move {
            from,
            to,
            piece,
            flag: MoveFlag::Quiet,
        }
    }

    pub fn new_with_flag(from: Square, to: Square, piece: Piece, flag: MoveFlag) -> Move {
        Move {
            from,
            to,
            piece,
            flag,
        }
    }

    pub fn set_flag(&mut self, flag: MoveFlag) {
        self.flag = flag;
    }
}
