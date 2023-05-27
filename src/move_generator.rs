use crate::board::Board;
use crate::types::{Bitboard, Color, Piece, Square};
use crate::utility::{pop_msb_1, FileBoundMask, RankPositionMask, MSB_BOARD};

/// Describes a move on the board and information related to that move
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    // TODO - add flags to help modify game state (new en passant square, change in castling availability, etc.)
}

struct Direction;
impl Direction {
    const N: isize = -8;
    const E: isize = 1;
    const S: isize = 8;
    const W: isize = -1;
    const NE: isize = Self::N + Self::E;
    const NW: isize = Self::N + Self::W;
    const SE: isize = Self::S + Self::E;
    const SW: isize = Self::S + Self::W;

    const DIAGONALS: [isize; 4] = [Self::NE, Self::NW, Self::SE, Self::SW];
    const STRAIGHTS: [isize; 4] = [Self::N, Self::E, Self::S, Self::W];
}

pub struct MoveGenerator {
    diagonal_attacks: [[Bitboard; 64]; 4],
    straight_attacks: [[Bitboard; 64]; 4],
}

/// Right-shifts the bitboard by the specified `amount` (if positive)
///
/// If `amount` is negative, a left-shift is applied with the same magnitude
fn negative_shift_right(board: Bitboard, amount: isize) -> Bitboard {
    if amount >= 0 {
        board >> amount
    } else {
        board << -amount
    }
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        /// Only needed temporarily, generates the attack vector for all squares in a particular direction
        fn calculate_attack_in_direction(direction_offset: isize) -> [Bitboard; 64] {
            let mut boards: [Bitboard; 64] = [0; 64];

            // go through each square in the board to fill it in
            for square in 0..64 {
                // generate the initial square the piece is on
                // and the square of the next attack, with the bitwise operation being handled for negative directions
                let mut attack: Bitboard = MSB_BOARD >> square;
                let mut next_attack: Bitboard = negative_shift_right(attack, direction_offset);

                // to tell if we are going off to the other side, the attack and next attack will be on the A and H file
                while ((attack & FileBoundMask::A) | (next_attack & FileBoundMask::H) != 0)
                    && ((attack & FileBoundMask::H) | (next_attack & FileBoundMask::A) != 0)
                {
                    // if the next attack is valid (and not wrapping to the other side of the board), we can now advance to the next attack (for the while loop)
                    attack = next_attack;
                    next_attack = negative_shift_right(attack, direction_offset);

                    // add this attack to the boards at the current square
                    boards[square] |= attack;
                }
            }

            boards
        }

        MoveGenerator {
            // generate diagonal attack directions
            diagonal_attacks: Direction::DIAGONALS.map(calculate_attack_in_direction),
            straight_attacks: Direction::STRAIGHTS.map(calculate_attack_in_direction),
        }
    }

    /// Generates a `Vec<Move>` containing all valid moves, given a board state
    ///
    /// Currently just pseudo-legal moves, checks are not considered
    pub fn generate_moves(&self, board: &Board) -> Vec<Move> {
        use Piece::*;

        let mut moves: Vec<Move> = Vec::new();

        // iterate through each type of piece
        for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
            // get the bitboard representing the pieces that can move of this type
            let mut pieces_board = board.active_piece_board(piece);

            // go through each position that this piece occurs in and pop it from the pieces bitboard
            while pieces_board != 0 {
                let from = pop_msb_1(&mut pieces_board);
                let position = MSB_BOARD >> from;

                // generate the correct move bitboard
                let mut moves_board = match piece {
                    Pawn => match board.active_color() {
                        Color::White => Self::generate_white_pawn_moves(
                            position,
                            board.active_color_board(),
                            board.inactive_color_board(),
                        ),
                        Color::Black => Self::generate_black_pawn_moves(
                            position,
                            board.active_color_board(),
                            board.inactive_color_board(),
                        ),
                    },
                    Knight => Self::generate_knight_moves(position, board.active_color_board()),
                    Bishop => self.generate_bishop_moves(
                        position,
                        board.active_color_board() | board.inactive_color_board(),
                        board.active_color_board(),
                    ), // TODO - generate sliding piece moves
                    Rook => self.generate_rook_moves(
                        position,
                        board.active_color_board() | board.inactive_color_board(),
                        board.active_color_board(),
                    ), // TODO - generate sliding piece moves
                    Queen => 0, // TODO - generate sliding piece moves
                    King => Self::generate_king_moves(position, board.active_color_board()),
                };

                // and similarly pop each bit from the bitboard, pushing a move to the list as we go
                while moves_board != 0 {
                    let to = pop_msb_1(&mut moves_board);
                    moves.push(Move { from, to, piece });
                }
            }
        }

        moves
    }

    // board move representation:
    // 1  4  6
    // 2 (K) 7
    // 3  5  8
    // moves 1,2,3 need to be bounds checked against A file
    // moves 6,7,8 need to be bounds checked against H file
    // moves don't need to be bounds checked against ranks, overflow will handle them
    fn generate_king_moves(king_position: Bitboard, same_color_pieces: Bitboard) -> Bitboard {
        // bounds check against files by bitwise AND king position with a file mask, where all bits in that file are 0
        // if the king is on that file, the king bit will disappear
        let king_position_not_a_file = king_position & FileBoundMask::A;
        let king_position_not_h_file = king_position & FileBoundMask::H;

        // first shift the king position in each direction, applying bounds checking when needed
        let moves: [Bitboard; 8] = [
            king_position_not_a_file << 9,
            king_position_not_a_file << 1,
            king_position_not_a_file >> 7,
            king_position << 8,
            king_position >> 8,
            king_position_not_h_file << 7,
            king_position_not_h_file >> 1,
            king_position_not_h_file >> 9,
        ];

        // bitwise OR all moves together, all 1's will appear in this bitboard
        let all_moves = moves.into_iter().fold(0, |curr, next| (curr | next));

        // bitwise AND all_moves with the negation of the same color pieces,
        // wherever there is a king move on top of the same color piece, 1 & !(1) => 1 & 0 => 0
        let valid_moves = all_moves & !same_color_pieces;

        valid_moves
    }

    // board move representation:
    // .  3  .  5  .
    // 1  .  .  .  7
    // .  . (N) .  .
    // 2  .  .  .  8
    // .  4  .  6  .
    // moves 1,2 need to be bounds checked against A and B file
    // moves 3,4 need to be bounds checked against A file
    // moves 5,6 need to be bounds checked against H file
    // moves 7,8 need to be bounds checked against G and H file
    // TODO - this method is verrrry similar to king moves, maybe some parts can be combined
    fn generate_knight_moves(knight_position: Bitboard, same_color_pieces: Bitboard) -> Bitboard {
        // bounds check against files
        let knight_position_not_a_file = knight_position & FileBoundMask::A;
        let knight_position_not_h_file = knight_position & FileBoundMask::H;
        let knight_position_not_ab_file = knight_position_not_a_file & FileBoundMask::B;
        let knight_position_not_gh_file = knight_position_not_h_file & FileBoundMask::G;

        // first shift the knight position in each L shape, applying bounds checking when needed
        let moves: [Bitboard; 8] = [
            knight_position_not_ab_file << 10,
            knight_position_not_ab_file >> 6,
            knight_position_not_a_file << 17,
            knight_position_not_a_file >> 15,
            knight_position_not_h_file << 15,
            knight_position_not_h_file >> 17,
            knight_position_not_gh_file << 6,
            knight_position_not_gh_file >> 10,
        ];

        // bitwise OR all moves together, all 1's will appear in this bitboard
        let all_moves = moves.into_iter().fold(0, |curr, next| (curr | next));

        // bitwise AND all_moves with the negation of the same color pieces
        let valid_moves = all_moves & !same_color_pieces;

        valid_moves
    }

    // board move representation:
    // .  2  .
    // 3  1  4
    // . (P) .
    // move 3 needs to be bounds checked against A file
    // move 4 needs to be bounds checked against H file
    // TODO - en passant moves
    fn generate_white_pawn_moves(
        pawn_position: Bitboard,
        white_pieces: Bitboard,
        black_pieces: Bitboard,
    ) -> Bitboard {
        // get squares where no pieces sit on
        let no_pieces = !white_pieces & !black_pieces;

        // pawn can move forward unless any color piece blocks its way
        let forward_move = (pawn_position << 8) & no_pieces;

        // pawn can double move forward if forward move was successful, pawn was on second rank (now third), and same rules apply with blocking pieces
        let double_move = ((forward_move & RankPositionMask::THIRD) << 8) & no_pieces;

        // for attacks to happen, an opposite colored piece has to be on the square
        let left_attack = (pawn_position & FileBoundMask::A) << 9;
        let right_attack = (pawn_position & FileBoundMask::H) << 7;
        let valid_attacks = (left_attack | right_attack) & black_pieces;

        // moves are combination of forward moves, double moves, and attack moves
        forward_move | double_move | valid_attacks
    }

    // board move representation:
    // . (P) .
    // 3  1  4
    // .  2 .
    // move 3 needs to be bounds checked against A file
    // move 4 needs to be bounds checked against H file
    // TODO - en passant moves
    fn generate_black_pawn_moves(
        pawn_position: Bitboard,
        black_pieces: Bitboard,
        white_pieces: Bitboard,
    ) -> Bitboard {
        // get squares where no pieces sit on
        let no_pieces = !white_pieces & !black_pieces;

        // pawn can move forward unless any color piece blocks its way
        let forward_move = (pawn_position >> 8) & no_pieces;

        // pawn can double move forward if forward move was successful, pawn was on second rank (now third), and same rules apply with blocking pieces
        let double_move = ((forward_move & RankPositionMask::SIXTH) >> 8) & no_pieces;

        // for attacks to happen, an opposite colored piece has to be on the square
        let left_attack = (pawn_position & FileBoundMask::A) >> 7;
        let right_attack = (pawn_position & FileBoundMask::H) >> 9;
        let valid_attacks = (left_attack | right_attack) & white_pieces;

        // moves are combination of forward moves, double moves, and attack moves
        forward_move | double_move | valid_attacks
    }

    /// Bishop move generation generally works as follows:
    ///
    /// 1. Index the pre-generated 2D array by the direction of attack and the square the bishop is on
    /// 2. Bitwise AND the attack ray and all pieces to find all pieces blocking this bishop
    /// 3. Find the index of the nearest blocker to the bishop and clip the attack off at that piece
    /// 4. Make sure the first blocker is not a piece of the same color, if it is remove that square
    fn generate_bishop_moves(
        &self,
        bishop_position: Bitboard,
        all_pieces: Bitboard,
        same_color_pieces: Bitboard,
    ) -> Bitboard {
        // get the square this bishop is on to index attack direction arrays
        let bishop_square = bishop_position.leading_zeros() as usize;

        let mut moves: Bitboard = 0;

        // go through the directions and attacks associated with each direction
        for (direction, attacks) in Direction::DIAGONALS.into_iter().zip(self.diagonal_attacks) {
            // by AND-ing the bishop's attack with all pieces, we get the pieces that block this attack
            let blocker_board = attacks[bishop_square] & all_pieces;

            let clipped_attack = if blocker_board == 0 {
                // if there are no pieces blocking, then the entire attack direction is kept
                attacks[bishop_square]
            } else {
                // else, find the first piece in the blocking direction
                let first_blocker = if direction > 0 {
                    // if the direction is southward, the first piece will be closest to the MSB
                    blocker_board.leading_zeros() as usize
                } else {
                    // else the first piece will be closest to the LSB (and subtract 63 because we need it in terms of MSB, not LSB)
                    63 - blocker_board.trailing_zeros() as usize
                };

                // finally, XOR the attack with the same direction attack from this first blocker to clip it off after the blocker
                attacks[bishop_square] ^ attacks[first_blocker]
            };

            // add this attack direction to the moves bitboard
            moves |= clipped_attack;
        }

        // since all pieces are used to find blockers, this bishop may be attacking a same-color piece
        // this AND will take the possibly invalid final move in the slide and see if it shares a space with a piece of the same color
        moves & !same_color_pieces
    }

    /// Nearly identical to bishop move generation, only change is that straight attacks are used instead of diagonals
    fn generate_rook_moves(
        &self,
        rook_position: Bitboard,
        all_pieces: Bitboard,
        same_color_pieces: Bitboard,
    ) -> Bitboard {
        // get the square this rook is on to index attack direction arrays
        let rook_square = rook_position.leading_zeros() as usize;

        let mut moves: Bitboard = 0;

        // go through the directions and attacks associated with each direction
        for (direction, attacks) in Direction::STRAIGHTS.into_iter().zip(self.straight_attacks) {
            // by AND-ing the rook's attack with all pieces, we get the pieces that block this attack
            let blocker_board = attacks[rook_square] & all_pieces;

            let clipped_attack = if blocker_board == 0 {
                // if there are no pieces blocking, then the entire attack direction is kept
                attacks[rook_square]
            } else {
                // else, find the first piece in the blocking direction
                let first_blocker = if direction > 0 {
                    // if the direction is southward, the first piece will be closest to the MSB
                    blocker_board.leading_zeros() as usize
                } else {
                    // else the first piece will be closest to the LSB (and subtract 63 because we need it in terms of MSB, not LSB)
                    63 - blocker_board.trailing_zeros() as usize
                };

                // finally, XOR the attack with the same direction attack from this first blocker to clip it off after the blocker
                attacks[rook_square] ^ attacks[first_blocker]
            };

            // add this attack direction to the moves bitboard
            moves |= clipped_attack;
        }

        // since all pieces are used to find blockers, this rook may be attacking a same-color piece
        // this AND will take the possibly invalid final move in the slide and see if it shares a space with a piece of the same color
        moves & !same_color_pieces
    }
}
