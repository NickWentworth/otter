use std::ops::{Index, IndexMut};

pub type Bitboard = u64;

pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
pub const NUM_PIECES: usize = 6;

impl Index<Piece> for [Bitboard] {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Piece> for [Bitboard] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

pub enum Color {
    White,
    Black,
}
pub const NUM_COLORS: usize = 2;

impl Index<Color> for [Bitboard] {
    type Output = Bitboard;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Color> for [Bitboard] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
