use crate::board::Board;
use crate::types::{Bitboard, Color, Piece, Square};
use crate::utility::{
    conditional_shift_right, pop_msb_1, FileBoundMask, RankPositionMask, MSB_BOARD,
};

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

type DirectionAttackPair = (isize, [Bitboard; 64]);

pub struct MoveGenerator {
    diagonal_attacks: [DirectionAttackPair; 4],
    straight_attacks: [DirectionAttackPair; 4],
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        // Only needed temporarily, generates the attack ray for all squares in a particular direction
        let calculate_attack_in_direction = |direction_offset: isize| -> DirectionAttackPair {
            let mut boards: [Bitboard; 64] = [0; 64];

            // go through each square in the board to fill it in
            for square in 0..64 {
                // generate the initial square the piece is on
                // and the square of the next attack, with the bitwise operation being handled for negative directions
                let mut attack: Bitboard = MSB_BOARD >> square;
                let mut next_attack: Bitboard = conditional_shift_right(attack, direction_offset);

                // to tell if we are going off to the other side, the attack and next attack will be on the A and H file
                while ((attack & FileBoundMask::A) | (next_attack & FileBoundMask::H) != 0)
                    && ((attack & FileBoundMask::H) | (next_attack & FileBoundMask::A) != 0)
                {
                    // if the next attack is valid (and not wrapping to the other side of the board), we can now advance to the next attack (for the while loop)
                    attack = next_attack;
                    next_attack = conditional_shift_right(attack, direction_offset);

                    // add this attack to the boards at the current square
                    boards[square] |= attack;
                }
            }

            (direction_offset, boards)
        };

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

        // fetch these once instead of generating for every piece
        let active_color = board.active_color();
        let en_passant = board.en_passant_square();
        let same_color = board.active_color_board();
        let oppositng_color = board.inactive_color_board();
        let both_colors = same_color | oppositng_color;

        // iterate through each type of piece
        for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
            // get the bitboard representing the pieces that can move of this type
            let mut pieces_board = board.active_piece_board(piece);

            // go through each position that this piece occurs in and pop it from the pieces bitboard
            while pieces_board != 0 {
                let from = pop_msb_1(&mut pieces_board);
                let piece_position = MSB_BOARD >> from;

                // generate the correct move bitboard
                let mut moves_board = match piece {
                    Pawn => match active_color {
                        Color::White => Self::generate_white_pawn_moves(
                            piece_position,
                            same_color,
                            oppositng_color,
                            en_passant,
                        ),
                        Color::Black => Self::generate_black_pawn_moves(
                            piece_position,
                            same_color,
                            oppositng_color,
                            en_passant,
                        ),
                    },
                    Knight => Self::generate_knight_moves(piece_position, same_color),
                    Bishop => Self::generate_sliding_moves(
                        piece_position,
                        both_colors,
                        same_color,
                        &self.diagonal_attacks,
                    ),
                    Rook => Self::generate_sliding_moves(
                        piece_position,
                        both_colors,
                        same_color,
                        &self.straight_attacks,
                    ),
                    Queen => {
                        Self::generate_sliding_moves(
                            piece_position,
                            both_colors,
                            same_color,
                            &self.diagonal_attacks,
                        ) | Self::generate_sliding_moves(
                            piece_position,
                            both_colors,
                            same_color,
                            &self.straight_attacks,
                        )
                    }
                    King => Self::generate_king_moves(piece_position, same_color),
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
    fn generate_white_pawn_moves(
        pawn_position: Bitboard,
        white_pieces: Bitboard,
        black_pieces: Bitboard,
        en_passant_square: Bitboard,
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
        let valid_attacks = (left_attack | right_attack) & (black_pieces | en_passant_square);

        // moves are combination of forward moves, double moves, and attack moves
        forward_move | double_move | valid_attacks
    }

    // board move representation:
    // . (P) .
    // 3  1  4
    // .  2 .
    // move 3 needs to be bounds checked against A file
    // move 4 needs to be bounds checked against H file
    fn generate_black_pawn_moves(
        pawn_position: Bitboard,
        black_pieces: Bitboard,
        white_pieces: Bitboard,
        en_passant_square: Bitboard,
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
        let valid_attacks = (left_attack | right_attack) & (white_pieces | en_passant_square);

        // moves are combination of forward moves, double moves, and attack moves
        forward_move | double_move | valid_attacks
    }

    /// Sliding piece move generation generally works as follows:
    ///
    /// 1. Index the pre-generated 2D array by the direction of attack and the square the attacking piece is on
    /// 2. Bitwise AND the attack ray and all pieces to find the pieces blocking the attacking piece
    /// 3. Find the index of the nearest blocker to the attacking piece and clip the attack off at that piece
    /// 4. Make sure the first blocker is not a piece of the same color, if it is remove that square
    fn generate_sliding_moves(
        piece_position: Bitboard,
        all_pieces: Bitboard,
        same_color_pieces: Bitboard,
        attacks: &[DirectionAttackPair],
    ) -> Bitboard {
        // get the square this bishop is on to index attack direction arrays
        let piece_square = piece_position.leading_zeros() as usize;

        let mut moves: Bitboard = 0;

        // go through the directions and attacks associated with each direction
        for (direction, attacks) in attacks {
            // by AND-ing the piece's attack with all pieces, we get the pieces that block this attack
            let blocker_board = attacks[piece_square] & all_pieces;

            let clipped_attack = if blocker_board == 0 {
                // if there are no pieces blocking, then the entire attack direction is kept
                attacks[piece_square]
            } else {
                // else, find the first piece in the blocking direction
                let first_blocker = if *direction > 0 {
                    // if the direction is southward, the first piece will be closest to the MSB
                    blocker_board.leading_zeros() as usize
                } else {
                    // else the first piece will be closest to the LSB (and subtract 63 because we need it in terms of MSB, not LSB)
                    63 - blocker_board.trailing_zeros() as usize
                };

                // finally, XOR the attack with the same direction attack from this first blocker to clip it off after the blocker
                attacks[piece_square] ^ attacks[first_blocker]
            };

            // add this attack direction to the moves bitboard
            moves |= clipped_attack;
        }

        // since all pieces are used to find blockers, this bishop may be attacking a same-color piece
        // this AND will take the possibly invalid final move in the slide and see if it shares a space with a piece of the same color
        moves & !same_color_pieces
    }
}
