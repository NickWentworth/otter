use crate::{
    board::Board,
    core::{Bitboard, Color, Piece, Square, BOARD_SIZE, PROMOTION_PIECES},
};

mod direction;
mod magic;
mod masks;
mod moves;

pub use moves::{Move, MoveFlag};

use direction::{
    BISHOP_MOVES, KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, PAWN_DOUBLE, PAWN_SINGLE, QUEEN_MOVES,
    ROOK_MOVES,
};
use magic::{BISHOP_MAGICS, ROOK_MAGICS};
use masks::{CastleMask, RankPositionMask};

pub struct MoveGenerator;
impl MoveGenerator {
    /// Generates a `Vec<Move>` containing all legal moves, given a board state
    pub fn generate_moves(board: &Board) -> Vec<Move> {
        use MoveFlag::*;
        use Piece::*;

        // firstly, create some masks to help filter out illegal moves

        // king can only move into safe squares not attacked by opposing pieces
        let king_board = board.active_piece_board(King);
        let king_square = king_board.get_first_square();

        let king_move_mask = Self::get_safe_king_squares(king_square, board);

        // other pieces (in the case of check) can either capture a checking piece or block it if it slides
        let (capture_mask, block_mask) = {
            let mut attackers = Bitboard::EMPTY;

            // get all attackers of the currently moving king by setting the king to different pieces
            // if the piece can attack an opposing piece of the same type, that means the king is attacked
            attackers |= Self::generate_sliding_attack(king_square, Bishop, board.all_pieces())
                & board.inactive_piece_board(Bishop);
            attackers |= Self::generate_sliding_attack(king_square, Rook, board.all_pieces())
                & board.inactive_piece_board(Rook);
            attackers |= Self::generate_sliding_attack(king_square, Queen, board.all_pieces())
                & board.inactive_piece_board(Queen);
            attackers |= KNIGHT_MOVES[king_square] & board.inactive_piece_board(Knight);
            attackers |=
                PAWN_ATTACKS[board.active_color()][king_square] & board.inactive_piece_board(Pawn);

            // based on how many pieces attack the king, there are different cases for movable squares
            match attackers.count_bits() {
                // nothing in check, no special masks needed
                0 => (Bitboard::FULL, Bitboard::FULL),

                // for a single check, other pieces can either capture the attacking piece or block it if it slides
                1 => (attackers, {
                    let attacker_square = attackers.get_first_square();
                    let attacker_piece = board.piece_at(attacker_square).unwrap();

                    if attacker_piece.is_sliding() {
                        Self::generate_sliding_attack_at_square(
                            king_square,
                            attacker_square,
                            attacker_piece,
                            board.all_pieces(),
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

        // find all pinned pieces and get a mask of their only legal moves
        let pin_masks = {
            // initially no pins, only will be there if set
            let mut masks = [Bitboard::FULL; BOARD_SIZE];

            // get a bitboard of all possible pinned friendly pieces by attacking in every direction from king square
            let king_attackable_pieces =
                Self::generate_sliding_attack(king_square, Queen, board.all_pieces())
                    & board.active_pieces();

            // for each opposing sliding piece, see if it attacks one of the possible pinned friendly pieces
            for opposing_square in board.inactive_pieces() {
                let opposing_piece = board.piece_at(opposing_square).unwrap();

                // only sliding pieces can create a pin
                if !opposing_piece.is_sliding() {
                    continue;
                }

                // get attackable pieces
                let opposing_attackable_pieces = Self::generate_sliding_attack(
                    opposing_square,
                    opposing_piece,
                    board.all_pieces(),
                ) & board.active_pieces();

                // and get any possible pinned pieces from this attacking opposing piece
                let possible_pins = opposing_attackable_pieces & king_attackable_pieces;

                // go through each possibly pinned piece and see if an attack can be generated through it
                for pinned_square in possible_pins {
                    let pinned_piece_position = Bitboard::shifted_board(pinned_square);

                    // try to get attack ray on the king, skipping through the pinned piece
                    let attack_through_pin = Self::generate_sliding_attack_at_square(
                        king_square,
                        opposing_square,
                        opposing_piece,
                        board.all_pieces() & !pinned_piece_position,
                    );

                    // if the attack is empty, it means the piece was not able to attack the king and there is no pin
                    // the pinned square must also be involved in the attack, otherwise the attack may just be a check with this piece off to the side
                    if !attack_through_pin.is_empty() && attack_through_pin.bit_at(pinned_square) {
                        // else, we set this square as pinned
                        masks[pinned_square] = attack_through_pin; // only allow it to move along the attack
                        masks[pinned_square].set_bit_at(opposing_square, true); // or capture the pinning piece
                    }
                }
            }

            masks
        };

        // now iterate through each type of piece, generating their moves
        let mut moves = Vec::new();

        for from_square in board.active_pieces() {
            let moving_piece = board.piece_at(from_square).unwrap();

            // piece is only allowed to move according to the pin mask
            let pin_mask = pin_masks[from_square];

            // pawn moves are wacky so generate these separately
            if moving_piece == Pawn {
                // pawn pushes
                let single_move =
                    PAWN_SINGLE[board.active_color()][from_square] & pin_mask & !board.all_pieces();

                // double move is only valid if single move isn't blocked
                let double_move = if single_move.is_empty() {
                    Bitboard::EMPTY
                } else {
                    PAWN_DOUBLE[board.active_color()][from_square] & pin_mask & !board.all_pieces()
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
                let normal_attacks = PAWN_ATTACKS[board.active_color()][from_square]
                    & capture_mask // pawn attack will only count as a capture
                    & pin_mask // and move according to pins
                    & board.inactive_pieces(); // and can only attack opposing pieces

                for to_square in normal_attacks {
                    let captured_piece = board.piece_at(to_square).unwrap();

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

                // finally, handle en passant attacks
                let en_passant_attack =
                    PAWN_ATTACKS[board.active_color()][from_square] & board.en_passant_board();

                // en passants can have hard-to-find pins
                // since they are uncommon we can just check if the king is in check after the move
                if !en_passant_attack.is_empty() {
                    // destination of our attacking pawn
                    let en_passant_destination = en_passant_attack.get_first_square();

                    // square that the opposing piece being taken is on
                    let en_passant_target = match board.active_color() {
                        Color::White => en_passant_destination + 8,
                        Color::Black => en_passant_destination - 8,
                    };

                    // temporarily move the pieces
                    let mut temp_all_pieces = board.all_pieces();
                    temp_all_pieces.set_bit_at(from_square, false); // remove moving piece
                    temp_all_pieces.set_bit_at(en_passant_destination, true); // move to destination
                    temp_all_pieces.set_bit_at(en_passant_target, false); // and delete target

                    // and check for king under attack
                    let king_under_attack = {
                        let mut attacked = false;

                        for opposing_square in board.inactive_pieces() {
                            let opposing_piece = board.piece_at(opposing_square).unwrap();

                            if opposing_piece.is_sliding() {
                                let attack_on_king = Self::generate_sliding_attack_at_square(
                                    king_square,
                                    opposing_square,
                                    opposing_piece,
                                    temp_all_pieces,
                                );

                                // if an attack on king exists, it would be in check
                                if !attack_on_king.is_empty() {
                                    attacked = true;
                                    break;
                                }
                            }
                        }

                        attacked
                    };

                    // if the king is not under attack, add the move
                    if !king_under_attack {
                        moves.push(Move {
                            from: from_square,
                            to: en_passant_destination,
                            piece: Pawn,
                            flag: EnPassantCapture(en_passant_target),
                        })
                    }
                }

                continue;
            }

            // regular attacking moves
            let attack_moves = match moving_piece {
                King => KING_MOVES[from_square] & king_move_mask,

                Knight => KNIGHT_MOVES[from_square] & (capture_mask | block_mask),

                Bishop | Rook | Queen => {
                    Self::generate_sliding_attack(from_square, moving_piece, board.all_pieces())
                        & (capture_mask | block_mask)
                }

                // easier to handle pawns elsewhere
                Pawn => unreachable!(),
            } & pin_mask // also must move according to pins
                & !board.active_pieces(); // and not into the same color pieces

            // iterate through legal moves and push into list
            for to_square in attack_moves {
                moves.push(Move {
                    from: from_square,
                    to: to_square,
                    piece: moving_piece,
                    flag: match board.piece_at(to_square) {
                        Some(captured_piece) => Capture(captured_piece),
                        None => Quiet,
                    },
                })
            }
        }

        // try to generate castling moves
        if board.active_kingside_rights() {
            // check if squares between king and rook are empty on the kingside
            if (CastleMask::KINGSIDE_EMPTY[board.active_color()] & board.all_pieces()).is_empty() {
                // and check that there are only safe squares to move along
                if (CastleMask::KINGSIDE_SAFE[board.active_color()] & !king_move_mask).is_empty() {
                    // if so, add the castle move
                    moves.push(Move {
                        from: king_square,
                        to: king_square + 2, // destination square is 2 to the right
                        piece: King,
                        flag: KingCastle,
                    })
                }
            }
        }

        if board.active_queenside_rights() {
            // check if squares between king and rook are empty on the queenside
            if (CastleMask::QUEENSIDE_EMPTY[board.active_color()] & board.all_pieces()).is_empty() {
                // and check that there are only safe squares to move along
                if (CastleMask::QUEENSIDE_SAFE[board.active_color()] & !king_move_mask).is_empty() {
                    // if so, add the castle move
                    moves.push(Move {
                        from: king_square,
                        to: king_square - 2, // destination square is 2 to the left
                        piece: King,
                        flag: QueenCastle,
                    })
                }
            }
        }

        moves
    }

    /// Determines if in the current board state, the active king is in check
    pub fn in_check(board: &Board) -> bool {
        use Piece::*;

        let king_position = board.active_piece_board(King);

        for square in board.inactive_pieces() {
            let piece = board.piece_at(square).unwrap();

            let opposing_attacks = match piece {
                King => Bitboard::EMPTY, // opposing king cannot put our king in check

                Knight => KNIGHT_MOVES[square],

                Pawn => PAWN_ATTACKS[board.inactive_color()][square],

                Bishop | Rook | Queen => {
                    Self::generate_sliding_attack(square, piece, board.all_pieces())
                }
            };

            if !(opposing_attacks & king_position).is_empty() {
                return true;
            }
        }

        false
    }

    /// Generates a board of all un-attacked squares that are safe for king to move into, including undefended opposing pieces
    fn get_safe_king_squares(king_square: Square, board: &Board) -> Bitboard {
        use Piece::*;
        let mut attack_board = Bitboard::EMPTY;

        let king_position = Bitboard::shifted_board(king_square);

        // go through all opposing pieces, popping one from the bitboard each iteration
        for square in board.inactive_pieces() {
            let piece = board.piece_at(square).unwrap();

            let current_piece_attack = match piece {
                King => KING_MOVES[square],
                Knight => KNIGHT_MOVES[square],
                Pawn => PAWN_ATTACKS[board.inactive_color()][square],

                // importantly, the king square is not taken into account in the attacked square generation for sliding pieces
                // if the king is attacked by a sliding piece, it should not be able to move backwards further into the piece's attack range
                // to fix this, the king square can be omitted and things will work as expected
                Rook | Bishop | Queen => Self::generate_sliding_attack(
                    square,
                    piece,
                    board.all_pieces() & !king_position,
                ),
            };

            attack_board |= current_piece_attack;
        }

        !attack_board
    }

    /// Helper function that generates the attacked square bitboard for a given sliding piece and square
    ///
    /// Does not remove the same color pieces being defended, but does clip them properly as expected
    fn generate_sliding_attack(piece_square: usize, piece: Piece, blockers: Bitboard) -> Bitboard {
        let mut moves = Bitboard::EMPTY;

        let attacks = match piece {
            Piece::Bishop => &(*BISHOP_MOVES),
            Piece::Rook => &(*ROOK_MOVES),
            Piece::Queen => &(*QUEEN_MOVES),
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
        target_square: Square,
        attacking_square: Square,
        attacking_piece: Piece,
        blockers: Bitboard,
    ) -> Bitboard {
        let attacks = match attacking_piece {
            Piece::Bishop => &(*BISHOP_MOVES),
            Piece::Rook => &(*ROOK_MOVES),
            Piece::Queen => &(*QUEEN_MOVES),
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
