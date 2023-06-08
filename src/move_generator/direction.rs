use crate::types::{Bitboard, Color, Square, BOARD_SIZE};
use std::collections::HashMap;

use super::masks::{FileBoundMask, RankPositionMask};

/// Describes the different directions of movement on the board as constants
pub struct Direction;
impl Direction {
    pub const N: isize = -8;
    pub const E: isize = 1;
    pub const S: isize = 8;
    pub const W: isize = -1;
    pub const NE: isize = Self::N + Self::E;
    pub const NW: isize = Self::N + Self::W;
    pub const SE: isize = Self::S + Self::E;
    pub const SW: isize = Self::S + Self::W;

    pub const DIAGONALS: [isize; 4] = [Self::NE, Self::NW, Self::SE, Self::SW];
    pub const STRAIGHTS: [isize; 4] = [Self::N, Self::E, Self::S, Self::W];
}

/// Generates a lookup table all king moves at each square
pub fn generate_king_moves() -> [Bitboard; BOARD_SIZE] {
    let mut boards = [Bitboard::EMPTY; BOARD_SIZE];

    for (square, board) in boards.iter_mut().enumerate() {
        // generate position and masked position bitboards
        let king_position = Bitboard::shifted_board(square);
        let king_position_a_file_masked = king_position & FileBoundMask::A;
        let king_position_h_file_masked = king_position & FileBoundMask::H;

        // board move representation:
        // 1  4  6
        // 2 (K) 7
        // 3  5  8

        // generate moves by bit shifting in each direction
        *board |= king_position_a_file_masked >> Direction::NW;
        *board |= king_position_a_file_masked >> Direction::W;
        *board |= king_position_a_file_masked >> Direction::SW;
        *board |= king_position >> Direction::N;
        *board |= king_position >> Direction::S;
        *board |= king_position_h_file_masked >> Direction::NE;
        *board |= king_position_h_file_masked >> Direction::E;
        *board |= king_position_h_file_masked >> Direction::SE;
    }

    boards
}

/// Generates a lookup table for knight moves at each square
pub fn generate_knight_moves() -> [Bitboard; BOARD_SIZE] {
    let mut boards = [Bitboard::EMPTY; BOARD_SIZE];

    for (square, board) in boards.iter_mut().enumerate() {
        // generate position and masked position bitboards
        let knight_position = Bitboard::shifted_board(square);
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

        // generate moves by bitshifting in each L shape
        *board |= knight_position_ab_file_masked >> Direction::NW + Direction::W;
        *board |= knight_position_ab_file_masked >> Direction::SW + Direction::W;
        *board |= knight_position_a_file_masked >> Direction::NW + Direction::N;
        *board |= knight_position_a_file_masked >> Direction::SW + Direction::S;
        *board |= knight_position_h_file_masked >> Direction::NE + Direction::N;
        *board |= knight_position_h_file_masked >> Direction::SE + Direction::S;
        *board |= knight_position_gh_file_masked >> Direction::NE + Direction::E;
        *board |= knight_position_gh_file_masked >> Direction::SE + Direction::E;
    }

    boards
}

/// Generates a lookup table for pawn attacks at each square, indexed by color
pub fn generate_pawn_attacks() -> HashMap<Color, [Bitboard; BOARD_SIZE]> {
    let mut white_boards = [Bitboard::EMPTY; BOARD_SIZE];
    let mut black_boards = [Bitboard::EMPTY; BOARD_SIZE];

    for (square, (white_board, black_board)) in
        white_boards.iter_mut().zip(&mut black_boards).enumerate()
    {
        // generate position and masked position bitboards
        let pawn_position = Bitboard::shifted_board(square);
        let pawn_position_a_file_masked = pawn_position & FileBoundMask::A;
        let pawn_position_h_file_masked = pawn_position & FileBoundMask::H;

        // board move representation:
        // white:       black:
        // .  .  .      . (P) .
        // 1  .  2      1  .  2
        // . (P) .      .  .  .

        // generate moves by shifting in each attacking direction (per color)
        *white_board |= pawn_position_a_file_masked >> Direction::NW;
        *white_board |= pawn_position_h_file_masked >> Direction::NE;

        *black_board |= pawn_position_a_file_masked >> Direction::SW;
        *black_board |= pawn_position_h_file_masked >> Direction::SE;
    }

    // build hashmap so we can index it by color
    let mut h = HashMap::new();
    h.insert(Color::White, white_boards);
    h.insert(Color::Black, black_boards);
    h
}

/// Generates a lookup table for pawn single moves at each square, indexed by color
pub fn generate_pawn_single_moves() -> HashMap<Color, [Bitboard; BOARD_SIZE]> {
    let mut white_boards = [Bitboard::EMPTY; BOARD_SIZE];
    let mut black_boards = [Bitboard::EMPTY; BOARD_SIZE];

    for (square, (white_board, black_board)) in
        white_boards.iter_mut().zip(&mut black_boards).enumerate()
    {
        // generate position bitboard
        let pawn_position = Bitboard::shifted_board(square);

        // board move representation:
        // white:       black:
        // .  .  .      . (P) .
        // .  1  .      .  1  .
        // . (P) .      .  .  .

        // generate moves by shifting in each moving direction (per color)
        *white_board |= pawn_position >> Direction::N;
        *black_board |= pawn_position >> Direction::S;
    }

    // build hashmap so we can index it by color
    let mut h = HashMap::new();
    h.insert(Color::White, white_boards);
    h.insert(Color::Black, black_boards);
    h
}

/// Generates a lookup table for pawn double moves at each square, indexed by color
pub fn generate_pawn_double_moves() -> HashMap<Color, [Bitboard; BOARD_SIZE]> {
    let mut white_boards = [Bitboard::EMPTY; BOARD_SIZE];
    let mut black_boards = [Bitboard::EMPTY; BOARD_SIZE];

    for (square, (white_board, black_board)) in
        white_boards.iter_mut().zip(&mut black_boards).enumerate()
    {
        // generate position bitboard
        let pawn_position = Bitboard::shifted_board(square);

        // board move representation:
        // white:       black:
        // .  1  .      . (P) .
        // .  .  .      .  .  .
        // . (P) .      .  1  .

        // generate moves by shifting in each moving direction (per color)
        *white_board |= (pawn_position & RankPositionMask::SECOND) >> Direction::N + Direction::N;
        *black_board |= (pawn_position & RankPositionMask::SEVENTH) >> Direction::S + Direction::S;
    }

    // build hashmap so we can index it by color
    let mut h = HashMap::new();
    h.insert(Color::White, white_boards);
    h.insert(Color::Black, black_boards);
    h
}

/// Generates a lookup table for the attack ray in a given direction (for sliding pieces )
pub fn generate_sliding_attacks(direction: isize) -> [Bitboard; BOARD_SIZE] {
    let mut boards = [Bitboard::EMPTY; BOARD_SIZE];

    for (square, board) in boards.iter_mut().enumerate() {
        // idea is to do the shift and see if we went over the bounds of the files
        // meaning one attack is on A file and other is on H
        let mut attack = Bitboard::shifted_board(square);

        while !((attack & FileBoundMask::A) | (attack >> direction & FileBoundMask::H)).is_empty()
            && !((attack & FileBoundMask::H) | (attack >> direction & FileBoundMask::A)).is_empty()
        {
            attack >>= direction;
            *board |= attack;
        }
    }

    boards
}
