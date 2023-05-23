use crate::board::Board;
use crate::types::{Bitboard, Color, Piece, Square};
use crate::utility::{FileBoundMask, RankPositionMask};

pub struct Move {
    from: Square,
    to: Square,
}

impl Move {
    fn new(from: Square, to: Square) -> Self {
        Move { from, to }
    }
}

// TODO - if two similar pieces can move to the same spot, data is lost
//        to fix this, separate each piece into its own bitboard and then generate its move
// generates a vector of all valid moves (currently just pseudo-legal ones, checks not considered)
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    let king_moves = generate_king_moves(
        board.active_piece_board(Piece::King),
        board.active_color_board(),
    );

    let knight_moves = generate_knight_moves(
        board.active_piece_board(Piece::Knight),
        board.active_color_board(),
    );

    let pawn_moves = match board.active_color() {
        Color::White => generate_white_pawn_moves(
            board.active_piece_board(Piece::Pawn),
            board.active_color_board(),
            board.inactive_color_board(),
        ),
        Color::Black => generate_black_pawn_moves(
            board.active_piece_board(Piece::Pawn),
            board.active_color_board(),
            board.inactive_color_board(),
        ),
    };

    // TODO - generate sliding piece moves
    // TODO - convert move bitboards to move structs

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
//    4  .  6  .
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
