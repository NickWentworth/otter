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
use masks::{CastleMask, RankPositionMask};
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
    pub fn generate_moves(&self, board: &Board) -> Vec<Move> {
        use Piece::*;

        let mut moves: Vec<Move> = Vec::new();

        // different variables representing the current board state for use in generating moves
        let info = board.get_board_info();

        // first, see if the king is currently in check and by how many pieces
        // to do this, we pretend the king's square is every other piece and see the opposing pieces it would attack
        // this allows us to accurately find all pieces attacking the king and get a count of how many pieces check it
        let mut attackers = Bitboard::EMPTY;

        let king_position = board.active_piece_board(King);
        let king_square = king_position.get_first_square() as usize;

        // get all attackers of the currently moving king
        attackers |= self.generate_sliding_attack_bitboard(king_square, Bishop, info.all_pieces)
            & board.inactive_piece_board(Bishop);
        attackers |= self.generate_sliding_attack_bitboard(king_square, Rook, info.all_pieces)
            & board.inactive_piece_board(Rook);
        attackers |= self.generate_sliding_attack_bitboard(king_square, Queen, info.all_pieces)
            & board.inactive_piece_board(Queen);
        attackers |= self.knight_moves[king_square] & board.inactive_piece_board(Knight);
        attackers |=
            self.pawn_attacks[&info.active_color][king_square] & board.inactive_piece_board(Pawn);

        // cases of different checks
        match attackers.count_bits() {
            // king not in check, generate moves as usual
            0 => {
                let mut active_pieces = info.same_pieces;
                while !active_pieces.is_empty() {
                    let from_square = active_pieces.pop_first_square();
                    let piece = info.piece_list[from_square as usize].unwrap(); // should be a piece here, safe to unwrap

                    let mut regular_moves = match piece {
                        King => {
                            // king cannot move into an attacked square
                            self.king_moves[from_square as usize]
                                & self.get_safe_king_squares(&info, from_square)
                        }

                        // TODO - handle pinned pieces
                        Knight => self.knight_moves[from_square as usize],

                        Pawn => self.pawn_attacks[&info.active_color][from_square as usize],

                        Bishop | Rook | Queen => self.generate_sliding_attack_bitboard(
                            from_square as usize,
                            piece,
                            info.all_pieces,
                        ),
                    };

                    // cannot move into a square of the same color
                    regular_moves &= !info.same_pieces;

                    // go through each move and add it as either a capture or quiet move
                    while !regular_moves.is_empty() {
                        let to_square = regular_moves.pop_first_square();

                        moves.push(Move {
                            from: from_square,
                            to: to_square,
                            piece,
                            flag: match info.piece_list[to_square as usize] {
                                // TODO - pawn capture can be a promotion in this case
                                Some(captured_piece) => MoveFlag::Capture(captured_piece),
                                None => MoveFlag::Quiet,
                            },
                        })
                    }
                }

                // TODO - other special moves (pawn pushes, castling)
            }

            // TODO - king is in check by one piece
            1 => todo!("implement single check"),

            // TODO - double check, only possible moves are king moves
            2 => todo!("implement double check"),

            // shouldn't be a case with 3+ pieces checking the king
            _ => panic!(),
        }

        // // iterate through each type of piece
        // for piece in ALL_PIECES {
        //     // get the bitboard representing the pieces that can move of this type
        //     let mut pieces_board = board.active_piece_board(piece);

        //     // go through each position that this piece occurs in and pop it from the pieces bitboard
        //     while !pieces_board.is_empty() {
        //         let piece_square = pieces_board.pop_first_square();
        //         let piece_position = Bitboard::shifted_board(piece_square);

        //         // TODO - test if adding to a pre-allocated move buffer is better than this method of extending vectors
        //         // and generate the moves for that piece
        //         moves.extend(match piece {
        //             // regular moving pieces
        //             King => self.generate_king_moves(piece_square, &info),
        //             Knight => self.generate_knight_moves(piece_square, &info),
        //             Pawn => self.generate_pawn_moves(piece_position, &info),

        //             // sliding pieces
        //             Bishop | Rook | Queen => {
        //                 self.generate_sliding_moves(piece_square, piece, &info)
        //             }
        //         })
        //     }
        // }

        moves
    }

    /// Given a board info, generates a board of all attacked squares that are unsafe for king to move into
    fn get_safe_king_squares(&self, info: &MoveGenBoardInfo, king_square: Square) -> Bitboard {
        use Piece::*;
        let mut attack_board = Bitboard::EMPTY;

        let mut opposing_pieces = info.opposing_pieces;
        let king_position = Bitboard::shifted_board(king_square);

        // go through all opposing pieces, popping one from the bitboard each iteration
        while !opposing_pieces.is_empty() {
            let square = opposing_pieces.pop_first_square() as usize;
            let piece = info.piece_list[square].unwrap();

            let current_piece_attack = match piece {
                King => self.king_moves[square],
                Knight => self.knight_moves[square],
                Pawn => self.pawn_attacks[&info.inactive_color][square],

                // importantly, the king square is not taken into account in the attacked square generation
                // this is because if the king is attacked by a sliding piece, it should not be able to move backwards further into the piece's attack range
                // to fix this, the king square can be omitted and things will work as expected
                Rook | Bishop | Queen => self.generate_sliding_attack_bitboard(
                    square,
                    piece,
                    info.all_pieces & !king_position,
                ),
            };

            attack_board |= current_piece_attack;
        }

        !attack_board
    }

    // TODO - prevent king from moving/castling into attacks
    fn generate_king_moves(&self, king_square: Square, info: &MoveGenBoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        let mut king_moves = self.king_moves[king_square as usize];

        // cannot move into squares occupied by the same color
        king_moves &= !info.same_pieces;

        // generate regular king moves
        while !king_moves.is_empty() {
            let to_square = king_moves.pop_first_square();

            moves.push(Move {
                from: king_square,
                to: to_square,
                piece: Piece::King,
                flag: match info.piece_list[to_square as usize] {
                    Some(piece) => MoveFlag::Capture(piece),
                    None => MoveFlag::Quiet,
                },
            });
        }

        // kingside castle check
        if info.king_castle_rights
            && (info.all_pieces & CastleMask::KINGSIDE[info.active_color]).is_empty()
        {
            moves.push(Move {
                from: king_square,
                to: king_square + 2,
                piece: Piece::King,
                flag: MoveFlag::KingCastle,
            });
        }

        // queenside castle check
        if info.queen_castle_rights
            && (info.all_pieces & CastleMask::QUEENSIDE[info.active_color]).is_empty()
        {
            moves.push(Move {
                from: king_square,
                to: king_square - 2,
                piece: Piece::King,
                flag: MoveFlag::QueenCastle,
            });
        }

        moves
    }

    fn generate_knight_moves(&self, knight_square: Square, info: &MoveGenBoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        let mut knight_moves = self.knight_moves[knight_square as usize];

        // cannot move into squares occupied by the same color
        knight_moves &= !info.same_pieces;

        // generate knight moves
        while !knight_moves.is_empty() {
            let to_square = knight_moves.pop_first_square();

            moves.push(Move {
                from: knight_square,
                to: to_square,
                piece: Piece::Knight,
                flag: match info.piece_list[to_square as usize] {
                    Some(piece) => MoveFlag::Capture(piece),
                    None => MoveFlag::Quiet,
                },
            });
        }

        moves
    }

    fn generate_pawn_moves(&self, pawn_position: Bitboard, info: &MoveGenBoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let from_square = pawn_position.get_first_square();

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
            let is_promotion = !(forward_move & RankPositionMask::PROMOTION).is_empty();

            if is_promotion {
                // if it is, go through all promotion pieces and add them in
                moves.extend(PROMOTION_PIECES.map(|promotion_piece| Move {
                    from: from_square,
                    to: forward_move.get_first_square(),
                    piece: Piece::Pawn,
                    flag: MoveFlag::Promotion(promotion_piece),
                }))
            } else {
                // if it isn't, just add a regular forward move in
                moves.push(Move {
                    from: from_square,
                    to: forward_move.get_first_square(),
                    piece: Piece::Pawn,
                    flag: MoveFlag::Quiet,
                })
            }
        }

        // check for a valid double move (based off of forward move)
        let double_move = ((forward_move & double_move_mask) >> direction) & info.no_pieces;
        if !double_move.is_empty() {
            moves.push(Move {
                from: from_square,
                to: double_move.get_first_square(),
                piece: Piece::Pawn,
                flag: MoveFlag::PawnDoubleMove(forward_move.get_first_square()),
            })
        }

        // get the moving color's pawn attacks at this square
        let attacks = self.pawn_attacks[&info.active_color][from_square as usize];

        // check for regular pawn attacks, not including en passant capture
        let mut regular_attacks = attacks & info.opposing_pieces;
        while !regular_attacks.is_empty() {
            let to_square = regular_attacks.pop_first_square();

            // cannot be an empty square, safe to unwrap
            let captured_piece = info.piece_list[to_square as usize].unwrap();

            // again check if this is a promotion move
            let is_promotion =
                !(Bitboard::shifted_board(to_square) & RankPositionMask::PROMOTION).is_empty();

            // check if this is a promotion rank here as well
            if is_promotion {
                // if it is, go through all promotion pieces and add them in as promotion attacks
                moves.extend(PROMOTION_PIECES.map(|promotion_piece| Move {
                    from: from_square,
                    to: to_square,
                    piece: Piece::Pawn,
                    flag: MoveFlag::CapturePromotion(captured_piece, promotion_piece),
                }))
            } else {
                // if it isn't, just add a regular attack move in
                moves.push(Move {
                    from: from_square,
                    to: to_square,
                    piece: Piece::Pawn,
                    flag: MoveFlag::Capture(captured_piece),
                })
            }
        }

        // check for attack on en passant square
        let en_passant_attack = attacks & info.en_passant;
        if !en_passant_attack.is_empty() {
            // will just be a single bit, no need to pop from bitboard
            let to_square = en_passant_attack.get_first_square();

            // opposing piece is located one square away from the attack in the opposite direction of this pawn's movement
            let opposing_piece_square = ((to_square as isize) - direction) as Square;

            moves.push(Move {
                from: from_square,
                to: to_square,
                piece: Piece::Pawn,
                flag: MoveFlag::EnPassantCapture(opposing_piece_square),
            })
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

        // build the bitboard properly clipped wherever there is a blocker
        let mut regular_moves =
            self.generate_sliding_attack_bitboard(piece_square as usize, piece, info.all_pieces);

        // since all pieces are used to find blockers, this bishop may be attacking a same-color piece
        // this AND will take the possibly invalid final move in the slide and see if it shares a space with a piece of the same color
        regular_moves &= !info.same_pieces;

        // now go through and add moves to vector
        while !regular_moves.is_empty() {
            let to_square = regular_moves.pop_first_square();

            moves.push(Move {
                from: piece_square,
                to: to_square,
                piece,
                flag: match info.piece_list[to_square as usize] {
                    Some(captured_piece) => MoveFlag::Capture(captured_piece),
                    None => MoveFlag::Quiet,
                },
            })
        }

        moves
    }

    /// Helper function that generates the attacked square bitboard for a given sliding piece at a square
    ///
    /// Does not remove the same color pieces being defended, but does clip them properly as expected
    fn generate_sliding_attack_bitboard(
        &self,
        piece_square: usize,
        piece: Piece,
        blockers: Bitboard,
    ) -> Bitboard {
        let mut moves = Bitboard::EMPTY;

        let attacks = match piece {
            Piece::Bishop => &self.diagonal_attacks,
            Piece::Rook => &self.straight_attacks,
            Piece::Queen => &self.all_attacks,
            _ => panic!("Pawn, Knight, or King are not sliding pieces!"),
        };

        // go through the directions and attacks associated with each direction
        for (direction, attacks) in attacks {
            // by AND-ing the piece's attack with all pieces, we get the pieces that block this attack
            let blocker_board = attacks[piece_square] & blockers;

            let clipped_attack = if blocker_board.is_empty() {
                // if there are no pieces blocking, then the entire attack direction is kept
                attacks[piece_square]
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
                attacks[piece_square] ^ attacks[first_blocker]
            };

            // add this attack direction to the moves bitboard
            moves |= clipped_attack;
        }

        moves
    }
}
