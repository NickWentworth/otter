use crate::{
    board::{Board, MoveGenBoardInfo},
    types::{Bitboard, Color, Piece, Square, BOARD_SIZE, PROMOTION_PIECES},
};
use std::collections::HashMap;

mod direction;
mod masks;
mod moves;

use direction::{
    generate_king_moves, generate_knight_moves, generate_pawn_attacks, generate_pawn_double_moves,
    generate_pawn_single_moves, generate_sliding_attacks, Direction,
};
use masks::RankPositionMask;
pub use moves::{Move, MoveFlag};

type DirectionAttackPair = (isize, [Bitboard; BOARD_SIZE]);

pub struct MoveGenerator {
    // simple move lookup boards
    king_moves: [Bitboard; BOARD_SIZE],
    knight_moves: [Bitboard; BOARD_SIZE],

    // pawn moves are split between single pushes, double pushes, and attacks
    pawn_single: HashMap<Color, [Bitboard; BOARD_SIZE]>,
    pawn_double: HashMap<Color, [Bitboard; BOARD_SIZE]>,
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

            pawn_single: generate_pawn_single_moves(),
            pawn_double: generate_pawn_double_moves(),
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

    /// Generates a `Vec<Move>` containing all legal moves, given a board state
    pub fn generate_moves(&self, board: &Board) -> Vec<Move> {
        use MoveFlag::*;
        use Piece::*;

        // different variables representing the current board state for use in generating moves
        let info = board.get_board_info();

        // firstly, create some masks to help filter out illegal moves

        // king can only move into safe squares not attacked by opposing pieces
        let king_board = board.active_piece_board(King);
        let king_square = king_board.get_first_square();

        let king_move_mask = self.get_safe_king_squares(king_square, &info);

        // other pieces (in the case of check) can either capture a checking piece or block it if it slides
        let (capture_mask, block_mask) = {
            let mut attackers = Bitboard::EMPTY;

            // get all attackers of the currently moving king by setting the king to different pieces
            // if the piece can attack an opposing piece of the same type, that means the king is attacked
            attackers |= self.generate_sliding_attack(king_square, Bishop, info.all_pieces)
                & board.inactive_piece_board(Bishop);
            attackers |= self.generate_sliding_attack(king_square, Rook, info.all_pieces)
                & board.inactive_piece_board(Rook);
            attackers |= self.generate_sliding_attack(king_square, Queen, info.all_pieces)
                & board.inactive_piece_board(Queen);
            attackers |= self.knight_moves[king_square] & board.inactive_piece_board(Knight);
            attackers |= self.pawn_attacks[&info.active_color][king_square]
                & board.inactive_piece_board(Pawn);

            // based on how many pieces attack the king, there are different cases for movable squares
            match attackers.count_bits() {
                // nothing in check, no special masks needed
                0 => (Bitboard::FULL, Bitboard::FULL),

                // for a single check, other pieces can either capture the attacking piece or block it if it slides
                1 => (attackers, {
                    let attacker_square = attackers.get_first_square();
                    let attacker_piece = info.piece_list[attacker_square].unwrap();

                    if attacker_piece.is_sliding() {
                        self.generate_sliding_attack_at_square(
                            king_square,
                            attacker_square,
                            attacker_piece,
                            info.all_pieces,
                        )
                    } else {
                        // cannot block a non-sliding attack
                        Bitboard::EMPTY
                    }
                }),

                // double check means only valid move is a king move
                2 => (Bitboard::EMPTY, Bitboard::EMPTY),

                // 3+ checks is impossible to have
                _ => panic!(),
            }
        };

        // TODO - check for pinned pieces

        // now iterate through each type of piece, generating their moves
        let mut moves = Vec::new();

        for from_square in info.same_pieces {
            let moving_piece = info.piece_list[from_square].unwrap();

            // pawn moves are wacky so generate these separately
            if moving_piece == Pawn {
                // pawn pushes
                let single_move =
                    self.pawn_single[&info.active_color][from_square] & !info.all_pieces;

                // double move is only valid if single move isn't blocked
                let double_move = if single_move.is_empty() {
                    Bitboard::EMPTY
                } else {
                    self.pawn_double[&info.active_color][from_square] & !info.all_pieces
                };

                // both single and double pushes can only block checks, not capture attackers
                // if a single move cannot block a check when a double move can, the double move is still legal (even if single is empty)

                // build pushing moves
                if !(single_move & block_mask).is_empty() {
                    let single_to_square = (single_move & block_mask).get_first_square();

                    if RankPositionMask::PROMOTION.bit_at(single_to_square) {
                        // if promotion, add all possible promotion pieces
                        for promotion_piece in PROMOTION_PIECES {
                            moves.push(Move {
                                from: from_square,
                                to: single_to_square,
                                piece: Pawn,
                                flag: Promotion(promotion_piece),
                            })
                        }
                    } else {
                        // else, just add single push
                        moves.push(Move {
                            from: from_square,
                            to: single_to_square,
                            piece: Pawn,
                            flag: Quiet,
                        })
                    }
                }

                if !(double_move & block_mask).is_empty() {
                    let single_to_square = single_move.get_first_square();
                    let double_to_square = (double_move & block_mask).get_first_square();

                    // add double push with correct square to be en passant-ed at
                    moves.push(Move {
                        from: from_square,
                        to: double_to_square,
                        piece: Pawn,
                        flag: PawnDoubleMove(single_to_square),
                    })
                }

                // now handle pawn attacks
                let mut normal_attacks = self.pawn_attacks[&info.active_color][from_square]
                    & capture_mask // pawn attack will only count as a capture
                    & info.opposing_pieces; // and can only attack opposing pieces

                if !normal_attacks.is_empty() {
                    let to_square = normal_attacks.pop_first_square();
                    let captured_piece = info.piece_list[to_square].unwrap();

                    if RankPositionMask::PROMOTION.bit_at(to_square) {
                        // if promotion, add all possible promotion pieces with the captured piece
                        for promotion_piece in PROMOTION_PIECES {
                            moves.push(Move {
                                from: from_square,
                                to: to_square,
                                piece: Pawn,
                                flag: CapturePromotion(captured_piece, promotion_piece),
                            })
                        }
                    } else {
                        // else, just add regular capture
                        moves.push(Move {
                            from: from_square,
                            to: to_square,
                            piece: Pawn,
                            flag: Capture(captured_piece),
                        })
                    }
                }

                // TODO - finally, handle en passant attacks

                continue;
            }

            // regular attacking moves
            let attack_moves = match moving_piece {
                King => self.king_moves[from_square] & king_move_mask,

                Knight => self.knight_moves[from_square] & (capture_mask | block_mask),

                Bishop | Rook | Queen => {
                    self.generate_sliding_attack(from_square, moving_piece, info.all_pieces)
                        & (capture_mask | block_mask)
                }

                // easier to handle pawns elsewhere
                Pawn => unreachable!(),
            } & !info.same_pieces;

            // iterate through legal moves and push into list
            for to_square in attack_moves {
                moves.push(Move {
                    from: from_square,
                    to: to_square,
                    piece: moving_piece,
                    flag: match info.piece_list[to_square] {
                        Some(captured_piece) => Capture(captured_piece),
                        None => Quiet,
                    },
                })
            }

            // TODO - generate castling moves
        }

        moves
    }

    /// Generates a board of all un-attacked squares that are safe for king to move into, including undefended opposing pieces
    fn get_safe_king_squares(&self, king_square: Square, info: &MoveGenBoardInfo) -> Bitboard {
        use Piece::*;
        let mut attack_board = Bitboard::EMPTY;

        let king_position = Bitboard::shifted_board(king_square);

        // go through all opposing pieces, popping one from the bitboard each iteration
        for square in info.opposing_pieces {
            let piece = info.piece_list[square].unwrap();

            let current_piece_attack = match piece {
                King => self.king_moves[square],
                Knight => self.knight_moves[square],
                Pawn => self.pawn_attacks[&info.inactive_color][square],

                // importantly, the king square is not taken into account in the attacked square generation for sliding pieces
                // if the king is attacked by a sliding piece, it should not be able to move backwards further into the piece's attack range
                // to fix this, the king square can be omitted and things will work as expected
                Rook | Bishop | Queen => {
                    self.generate_sliding_attack(square, piece, info.all_pieces & !king_position)
                }
            };

            attack_board |= current_piece_attack;
        }

        !attack_board
    }

    /// Helper function that generates the attacked square bitboard for a given sliding piece and square
    ///
    /// Does not remove the same color pieces being defended, but does clip them properly as expected
    fn generate_sliding_attack(
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
                };

                // finally, XOR the attack with the same direction attack from this first blocker to clip it off after the blocker
                attacks[piece_square] ^ attacks[first_blocker]
            };

            // add this attack direction to the moves bitboard
            moves |= clipped_attack;
        }

        moves
    }

    /// Similar to the function that generates an entire sliding attack, but this only generates the attack in the direction targeting the given target square
    ///
    /// Returns an empty board if no such attack exists
    fn generate_sliding_attack_at_square(
        &self,
        target_square: Square,
        attacking_square: Square,
        attacking_piece: Piece,
        blockers: Bitboard,
    ) -> Bitboard {
        let attacks = match attacking_piece {
            Piece::Bishop => &self.diagonal_attacks,
            Piece::Rook => &self.straight_attacks,
            Piece::Queen => &self.all_attacks,
            _ => panic!("Pawn, Knight, or King are not sliding pieces!"),
        };

        for (direction, attacks) in attacks {
            let blocker_board = attacks[attacking_square] & blockers;

            // if there are no pieces blocking this direction, then the target square can't possibly be being attacked
            if !blocker_board.is_empty() {
                // else, find the first piece in the blocking direction
                let first_blocker = if *direction > 0 {
                    // if the direction is southward, the first piece will be closest to the MSB
                    blocker_board.get_first_square()
                } else {
                    // else the first piece will be closest to the LSB (and subtract 63 because we need it in terms of MSB, not LSB)
                    blocker_board.get_last_square()
                };

                // if the first blocker is the target square, we have found the attack on the target
                if first_blocker == target_square {
                    // as usual, XOR the attack with the same direction attack from the first blocker to clip it off after the blocker
                    return attacks[attacking_square] ^ attacks[first_blocker];
                }
            };
        }

        // if the target square is not attacked, just return an empty bitboard
        Bitboard::EMPTY
    }
}
