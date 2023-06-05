use crate::{
    board::{Board, MoveGenBoardInfo},
    types::{Bitboard, Color, Piece, Square, ALL_PIECES, BOARD_SIZE, PROMOTION_PIECES},
};
use std::collections::HashMap;

mod direction;
mod masks;
mod moves;

use direction::{
    generate_king_moves, generate_knight_moves, generate_pawn_attacks, generate_sliding_attacks,
    Direction,
};
use masks::{CastleMask, FileBoundMask, RankPositionMask};
pub use moves::{Move, MoveFlag};

type DirectionAttackPair = (isize, [Bitboard; BOARD_SIZE]);

pub struct MoveGenerator {
    // simple move lookup boards
    king_moves: [Bitboard; BOARD_SIZE],
    knight_moves: [Bitboard; BOARD_SIZE],
    pawn_attacks: HashMap<Color, [Bitboard; BOARD_SIZE]>,

    // sliding move lookup boards
    diagonal_attacks: Vec<DirectionAttackPair>,
    straight_attacks: Vec<DirectionAttackPair>,
    all_attacks: Vec<DirectionAttackPair>,
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator {
            king_moves: generate_king_moves(),
            knight_moves: generate_knight_moves(),
            pawn_attacks: generate_pawn_attacks(),

            diagonal_attacks: Direction::DIAGONALS
                .map(|direction| (direction, generate_sliding_attacks(direction)))
                .to_vec(),
            straight_attacks: Direction::STRAIGHTS
                .map(|direction| (direction, generate_sliding_attacks(direction)))
                .to_vec(),
            all_attacks: [Direction::DIAGONALS, Direction::STRAIGHTS]
                .concat()
                .into_iter()
                .map(|direction| (direction, generate_sliding_attacks(direction)))
                .collect(),
        }
    }

    /// Generates a `Vec<Move>` containing all valid moves, given a board state
    ///
    /// Currently just pseudo-legal moves, checks are not considered
    pub fn generate_moves(&self, board: &Board) -> Vec<Move> {
        use Piece::*;

        let mut moves: Vec<Move> = Vec::new();

        // fetch these once instead of generating for every piece
        let info = board.get_board_info();

        // iterate through each type of piece
        for piece in ALL_PIECES {
            // get the bitboard representing the pieces that can move of this type
            let mut pieces_board = board.active_piece_board(piece);

            // go through each position that this piece occurs in and pop it from the pieces bitboard
            while !pieces_board.is_empty() {
                let piece_square = pieces_board.pop_first_square();
                let piece_position = Bitboard::shifted_board(piece_square);

                // TODO - test if adding to a pre-allocated move buffer is better than this method of extending vectors
                // and generate the moves for that piece
                moves.extend(match piece {
                    // regular moving pieces
                    King => self.generate_king_moves(piece_square, &info),
                    Knight => self.generate_knight_moves(piece_square, &info),
                    Pawn => self.generate_pawn_moves(piece_position, &info),

                    // sliding pieces
                    Bishop | Rook | Queen => {
                        self.generate_sliding_moves(piece_square, piece, &info)
                    }
                })
            }
        }

        moves
    }

    // TODO - prevent king from moving/castling into attacks
    fn generate_king_moves(&self, king_square: Square, info: &MoveGenBoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        let mut king_moves = self.king_moves[king_square as usize];

        // cannot move into squares occupied by the same color
        king_moves &= !info.same_pieces;

        // TODO - this is a common pattern in the move generation for different pieces, can likely be turned into a function
        while !king_moves.is_empty() {
            let to = king_moves.pop_first_square();
            let mut m = Move::new(king_square, to, Piece::King);

            // if an opposing piece is on this square, add a capture flag to it
            if let Some(piece) = info.piece_list[to as usize] {
                m.set_flag(MoveFlag::Capture(piece));
            }

            moves.push(m);
        }

        // kingside castle check
        if info.king_castle_rights
            && (info.all_pieces & CastleMask::KINGSIDE[info.active_color]).is_empty()
        {
            moves.push(Move::new_with_flag(
                king_square,
                king_square + 2,
                Piece::King,
                MoveFlag::KingCastle,
            ));
        }

        // queenside castle check
        if info.queen_castle_rights
            && (info.all_pieces & CastleMask::QUEENSIDE[info.active_color]).is_empty()
        {
            moves.push(Move::new_with_flag(
                king_square,
                king_square - 2,
                Piece::King,
                MoveFlag::QueenCastle,
            ));
        }

        moves
    }

    fn generate_knight_moves(&self, knight_square: Square, info: &MoveGenBoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        let mut knight_moves = self.knight_moves[knight_square as usize];

        // cannot move into squares occupied by the same color
        knight_moves &= !info.same_pieces;

        while !knight_moves.is_empty() {
            let to = knight_moves.pop_first_square();
            let mut m = Move::new(knight_square, to, Piece::Knight);

            // if an opposing piece is on this square, add a capture flag to it
            if let Some(piece) = info.piece_list[to as usize] {
                m.set_flag(MoveFlag::Capture(piece));
            }

            moves.push(m);
        }

        moves
    }

    fn generate_pawn_moves(&self, pawn_position: Bitboard, info: &MoveGenBoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let from = pawn_position.get_first_square();

        // board move representation:
        // white:       black:
        // .  2  .      . (P) .
        // 3  1  4      3  1  4
        // . (P) .      .  2  .

        // based on the color, pawn moving direction and rank for double moves are different
        let (direction, double_move_mask) = match info.active_color {
            Color::White => (Direction::N, RankPositionMask::THIRD),
            Color::Black => (Direction::S, RankPositionMask::SIXTH),
        };

        // check for a valid forward move
        let forward_move = (pawn_position >> direction) & info.no_pieces;
        if !forward_move.is_empty() {
            // check to see if the move is a promotion move
            if (forward_move & RankPositionMask::PROMOTION).is_empty() {
                // if it isn't, just add a regular forward move in
                moves.push(Move::new(
                    from,
                    forward_move.get_first_square(),
                    Piece::Pawn,
                ))
            } else {
                // else, go through all promotion pieces and add them in
                for promotion_piece in PROMOTION_PIECES {
                    moves.push(Move::new_with_flag(
                        from,
                        forward_move.get_first_square(),
                        Piece::Pawn,
                        MoveFlag::Promotion(promotion_piece),
                    ))
                }
            }
        }

        // check for a valid double move (based off of forward move)
        let double_move = ((forward_move & double_move_mask) >> direction) & info.no_pieces;
        if !double_move.is_empty() {
            moves.push(Move::new_with_flag(
                from,
                double_move.get_first_square(),
                Piece::Pawn,
                MoveFlag::PawnDoubleMove(forward_move.get_first_square()),
            ))
        }

        // get the moving color's pawn attacks at this square
        let attacks = self.pawn_attacks[&info.active_color][from as usize];

        // check for regular pawn attacks, not including en passant capture
        let mut regular_attacks = attacks & info.opposing_pieces;
        while !regular_attacks.is_empty() {
            let to = regular_attacks.pop_first_square();

            // cannot be an empty square, safe to unwrap
            let captured_piece = info.piece_list[to as usize].unwrap();

            // check if this is a promotion rank here as well
            if (Bitboard::shifted_board(to) & RankPositionMask::PROMOTION).is_empty() {
                // if it isn't, just add a regular attack move in
                moves.push(Move::new_with_flag(
                    from,
                    to,
                    Piece::Pawn,
                    MoveFlag::Capture(captured_piece),
                ))
            } else {
                // else, go through all promotion pieces and add them in as promotion attacks
                for promotion_piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                    moves.push(Move::new_with_flag(
                        from,
                        to,
                        Piece::Pawn,
                        MoveFlag::CapturePromotion(captured_piece, promotion_piece),
                    ))
                }
            }
        }

        // check for attack on en passant square
        let en_passant_attack = attacks & info.en_passant;
        if !en_passant_attack.is_empty() {
            // will just be a single bit, no need to pop from bitboard
            let to = en_passant_attack.get_first_square();

            // opposing piece is located one square away from the attack in the opposite direction of this pawn's movement
            let opposing_piece_square = ((to as isize) - direction) as Square;
            moves.push(Move::new_with_flag(
                from,
                to,
                Piece::Pawn,
                MoveFlag::EnPassantCapture(opposing_piece_square),
            ))
        }

        moves
    }

    // Sliding piece move generation generally works as follows:
    //
    // 1. Index the pre-generated 2D array by the direction of attack and the square the attacking piece is on
    // 2. Bitwise AND the attack ray and all pieces to find the pieces blocking the attacking piece
    // 3. Find the index of the nearest blocker to the attacking piece and clip the attack off at that piece
    // 4. Make sure the first blocker is not a piece of the same color, if it is remove that square
    //
    // From this, we have a bitboard that contains moves that are either quiet or captures
    fn generate_sliding_moves(
        &self,
        piece_square: Square,
        piece: Piece,
        info: &MoveGenBoardInfo,
    ) -> Vec<Move> {
        // get the square this bishop is on to index attack direction arrays
        let mut moves: Vec<Move> = Vec::new();

        let mut regular_moves = Bitboard::EMPTY;

        // TODO - just initially generate these as vectors, they aren't being mutated so accessing isn't any faster
        let attacks = match piece {
            Piece::Bishop => &self.diagonal_attacks,
            Piece::Rook => &self.straight_attacks,
            Piece::Queen => &self.all_attacks,
            _ => panic!("Pawn, Knight, or King are not sliding pieces!"),
        };

        // go through the directions and attacks associated with each direction
        for (direction, attacks) in attacks {
            // by AND-ing the piece's attack with all pieces, we get the pieces that block this attack
            let blocker_board = attacks[piece_square as usize] & info.all_pieces;

            let clipped_attack = if blocker_board.is_empty() {
                // if there are no pieces blocking, then the entire attack direction is kept
                attacks[piece_square as usize]
            } else {
                // else, find the first piece in the blocking direction
                let first_blocker = if *direction > 0 {
                    // if the direction is southward, the first piece will be closest to the MSB
                    blocker_board.get_first_square()
                } else {
                    // else the first piece will be closest to the LSB (and subtract 63 because we need it in terms of MSB, not LSB)
                    blocker_board.get_last_square()
                } as usize;

                // finally, XOR the attack with the same direction attack from this first blocker to clip it off after the blocker
                attacks[piece_square as usize] ^ attacks[first_blocker]
            };

            // add this attack direction to the moves bitboard
            regular_moves |= clipped_attack;
        }

        // since all pieces are used to find blockers, this bishop may be attacking a same-color piece
        // this AND will take the possibly invalid final move in the slide and see if it shares a space with a piece of the same color
        regular_moves &= !info.same_pieces;

        // now go through and add moves to vector
        while !regular_moves.is_empty() {
            let to = regular_moves.pop_first_square();
            let mut m = Move::new(piece_square, to, piece);

            // if an opposing piece is on this square, add a capture flag to it
            if let Some(piece) = info.piece_list[to as usize] {
                m.set_flag(MoveFlag::Capture(piece));
            }

            moves.push(m);
        }

        moves
    }
}
