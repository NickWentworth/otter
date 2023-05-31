use crate::bitboard::{Bitboard, Square};
use crate::board::Board;
use crate::types::{Color, Piece};
use crate::utility::{CastleMask, FileBoundMask, RankPositionMask};

pub enum MoveFlag {
    Quiet,                          // nothing special, regular move that doesn't have any flags
    Capture(Piece),                 // opponent piece that was captured
    Promotion(Piece),               // pawn was promoted into a piece
    CapturePromotion(Piece, Piece), // opponent piece that was captured as well as the piece promoted into
    PawnDoubleMove(Square),         // pawn double moved and stores the en passant square
    KingCastle,                     // kingside castle
    QueenCastle,                    // queenside castle
}

/// Describes a move on the board and information related to that move
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

struct BoardInfo<'a> {
    pub active_color: Color,
    pub inactive_color: Color,

    pub same_pieces: Bitboard,
    pub oppposing_pieces: Bitboard,
    pub all_pieces: Bitboard,
    pub no_pieces: Bitboard,

    pub en_passant: Bitboard,
    pub king_castle_rights: bool,
    pub queen_castle_rights: bool,

    pub board: &'a Board,
}

pub struct MoveGenerator {
    diagonal_attacks: [DirectionAttackPair; 4],
    straight_attacks: [DirectionAttackPair; 4],
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        // Only needed temporarily, generates the attack ray for all squares in a particular direction
        let calculate_attack_in_direction = |direction_offset: isize| -> DirectionAttackPair {
            let mut boards: [Bitboard; 64] = [Bitboard::EMPTY; 64];

            // go through each square in the board to fill it in
            for square in 0..=63 {
                // generate the initial square the piece is on
                // and the square of the next attack, with the bitwise operation being handled for negative directions
                let mut attack: Bitboard = Bitboard::shifted_board(square);
                let mut next_attack: Bitboard = attack >> direction_offset;

                // to tell if we are going off to the other side, the attack and next attack will be on the A and H file
                while !((attack & FileBoundMask::A) | (next_attack & FileBoundMask::H)).is_empty()
                    && !((attack & FileBoundMask::H) | (next_attack & FileBoundMask::A)).is_empty()
                {
                    // if the next attack is valid (and not wrapping to the other side of the board), we can now advance to the next attack (for the while loop)
                    attack = next_attack;
                    next_attack = attack >> direction_offset;

                    // add this attack to the boards at the current square
                    boards[square as usize] |= attack;
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

        // TODO - this could probably be returned from a board method
        // fetch these once instead of generating for every piece
        let info = BoardInfo {
            active_color: board.active_color(),
            inactive_color: board.active_color().opposite(),

            same_pieces: board.active_color_board(),
            oppposing_pieces: board.inactive_color_board(),
            all_pieces: board.active_color_board() | board.inactive_color_board(),
            no_pieces: !(board.active_color_board() | board.inactive_color_board()),

            en_passant: board.en_passant_square(),
            king_castle_rights: board.active_castling_rights().0,
            queen_castle_rights: board.active_castling_rights().1,

            board,
        };

        // iterate through each type of piece
        for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
            // get the bitboard representing the pieces that can move of this type
            let mut pieces_board = board.active_piece_board(piece);

            // go through each position that this piece occurs in and pop it from the pieces bitboard
            while !pieces_board.is_empty() {
                let from = pieces_board.pop_first_square();
                let piece_position = Bitboard::shifted_board(from);

                // TODO - test if adding to a pre-allocated move buffer is better than this method of extending vectors
                // and generate the moves for that piece
                moves.extend(match piece {
                    // regular moving pieces
                    King => Self::generate_king_moves(piece_position, &info),
                    Knight => Self::generate_knight_moves(piece_position, &info),
                    Pawn => Self::generate_pawn_moves(piece_position, &info),

                    // sliding pieces
                    Bishop | Rook | Queen => {
                        self.generate_sliding_moves(piece_position, piece, &info)
                    }
                })
            }
        }

        moves
    }

    // TODO - prevent king from moving/castling into attacks
    fn generate_king_moves(king_position: Bitboard, info: &BoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let from = king_position.get_first_square();

        let king_position_a_file_masked = king_position & FileBoundMask::A;
        let king_position_h_file_masked = king_position & FileBoundMask::H;

        // board move representation:
        // 1  4  6
        // 2 (K) 7
        // 3  5  8

        // generate regular moves by bit shifting in each direction
        let mut regular_moves = Bitboard::EMPTY;
        regular_moves |= king_position_a_file_masked >> Direction::NW;
        regular_moves |= king_position_a_file_masked >> Direction::W;
        regular_moves |= king_position_a_file_masked >> Direction::SW;
        regular_moves |= king_position >> Direction::N;
        regular_moves |= king_position >> Direction::S;
        regular_moves |= king_position_h_file_masked >> Direction::NE;
        regular_moves |= king_position_h_file_masked >> Direction::E;
        regular_moves |= king_position_h_file_masked >> Direction::SE;

        // cannot move into squares occupied by the same color
        regular_moves &= !info.same_pieces;

        // TODO - this is a common pattern in the move generation for different pieces, can likely be turned into a function
        while !regular_moves.is_empty() {
            let to = regular_moves.pop_first_square();
            let mut m = Move::new(from, to, Piece::King);

            // if an opposing piece is on this square, add a capture flag to it
            if let Some(piece) = info.board.piece_at_square(to, info.inactive_color) {
                m.set_flag(MoveFlag::Capture(piece));
            }

            moves.push(m);
        }

        // kingside castle check
        if info.king_castle_rights
            && (info.all_pieces & CastleMask::KINGSIDE[info.active_color]).is_empty()
        {
            moves.push(Move::new_with_flag(
                from,
                from + 2,
                Piece::King,
                MoveFlag::KingCastle,
            ));
        }

        // queenside castle check
        if info.queen_castle_rights
            && (info.all_pieces & CastleMask::QUEENSIDE[info.active_color]).is_empty()
        {
            moves.push(Move::new_with_flag(
                from,
                from - 2,
                Piece::King,
                MoveFlag::QueenCastle,
            ));
        }

        moves
    }

    // TODO - this method is verrrry similar to king moves, maybe some parts can be combined
    fn generate_knight_moves(knight_position: Bitboard, info: &BoardInfo) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let from = knight_position.get_first_square();

        let knight_position_a_file_masked = knight_position & FileBoundMask::A;
        let knight_position_h_file_masked = knight_position & FileBoundMask::H;
        let knight_position_ab_file_masked = knight_position_a_file_masked & FileBoundMask::B;
        let knight_position_gh_file_masked = knight_position_h_file_masked & FileBoundMask::G;

        // board move representation:
        // .  3  .  5  .
        // 1  .  .  .  7
        // .  . (N) .  .
        // 2  .  .  .  8
        // .  4  .  6  .

        // generate regular moves by bitshifting in each L shape
        let mut regular_moves = Bitboard::EMPTY;
        regular_moves |= knight_position_ab_file_masked >> Direction::NW + Direction::W;
        regular_moves |= knight_position_ab_file_masked >> Direction::SW + Direction::W;
        regular_moves |= knight_position_a_file_masked >> Direction::NW + Direction::N;
        regular_moves |= knight_position_a_file_masked >> Direction::SW + Direction::S;
        regular_moves |= knight_position_h_file_masked >> Direction::NE + Direction::N;
        regular_moves |= knight_position_h_file_masked >> Direction::SE + Direction::S;
        regular_moves |= knight_position_gh_file_masked >> Direction::NE + Direction::E;
        regular_moves |= knight_position_gh_file_masked >> Direction::SE + Direction::E;

        // cannot move into squares occupied by the same color
        regular_moves &= !info.same_pieces;

        while !regular_moves.is_empty() {
            let to = regular_moves.pop_first_square();
            let mut m = Move::new(from, to, Piece::Knight);

            // if an opposing piece is on this square, add a capture flag to it
            if let Some(piece) = info.board.piece_at_square(to, info.inactive_color) {
                m.set_flag(MoveFlag::Capture(piece));
            }

            moves.push(m);
        }

        moves
    }

    fn generate_pawn_moves(pawn_position: Bitboard, info: &BoardInfo) -> Vec<Move> {
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
                for promotion_piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
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

        // check for attacks in diagonal directions
        let left_attack = (pawn_position & FileBoundMask::A) >> (direction - 1);
        let right_attack = (pawn_position & FileBoundMask::H) >> (direction + 1);
        let mut attacks = (left_attack | right_attack) & (info.oppposing_pieces | info.en_passant);

        while !attacks.is_empty() {
            let to = attacks.pop_first_square();

            // cannot be an empty square, safe to unwrap
            let captured_piece = info.board.piece_at_square(to, info.inactive_color).unwrap();

            // check if this is a promotion rank here as well
            if (forward_move & RankPositionMask::PROMOTION).is_empty() {
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
        piece_position: Bitboard,
        piece: Piece,
        info: &BoardInfo,
    ) -> Vec<Move> {
        // get the square this bishop is on to index attack direction arrays
        let mut moves: Vec<Move> = Vec::new();
        let from = piece_position.get_first_square();
        let piece_square = from as usize;

        let mut regular_moves = Bitboard::EMPTY;

        // TODO - just initially generate these as vectors, they aren't being mutated so accessing isn't any faster
        let attacks = match piece {
            Piece::Bishop => [self.diagonal_attacks].concat(),
            Piece::Rook => [self.straight_attacks].concat(),
            Piece::Queen => [self.diagonal_attacks, self.straight_attacks].concat(),
            _ => panic!("Pawn, Knight, or King are not sliding pieces!"),
        };

        // go through the directions and attacks associated with each direction
        for (direction, attacks) in attacks {
            // by AND-ing the piece's attack with all pieces, we get the pieces that block this attack
            let blocker_board = attacks[piece_square] & info.all_pieces;

            let clipped_attack = if blocker_board.is_empty() {
                // if there are no pieces blocking, then the entire attack direction is kept
                attacks[piece_square]
            } else {
                // else, find the first piece in the blocking direction
                let first_blocker = if direction > 0 {
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
            regular_moves |= clipped_attack;
        }

        // since all pieces are used to find blockers, this bishop may be attacking a same-color piece
        // this AND will take the possibly invalid final move in the slide and see if it shares a space with a piece of the same color
        regular_moves &= !info.same_pieces;

        // now go through and add moves to vector
        while !regular_moves.is_empty() {
            let to = regular_moves.pop_first_square();
            let mut m = Move::new(from, to, piece);

            // if an opposing piece is on this square, add a capture flag to it
            if let Some(piece) = info.board.piece_at_square(to, info.inactive_color) {
                m.set_flag(MoveFlag::Capture(piece));
            }

            moves.push(m);
        }

        moves
    }
}
