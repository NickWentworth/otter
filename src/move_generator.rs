use crate::types::{Bitboard, FileBoundMask};

// board move representation:
// 1  4  6
// 2 (K) 7
// 3  5  8
// moves 1,2,3 need to be bounds checked against A file
// moves 6,7,8 need to be bounds checked against H file
// moves don't need to be bounds checked against ranks, overflow will handle them
pub fn generate_king_moves(king_position: Bitboard, same_color_pieces: Bitboard) -> Bitboard {
    // bounds check against files by bitwise AND king position with a file mask, where all bits in that file are 0
    // if the king is on that file, the king bit will disappear
    let king_position_not_a_file = king_position & (FileBoundMask::A as Bitboard);
    let king_position_not_h_file = king_position & (FileBoundMask::H as Bitboard);

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
