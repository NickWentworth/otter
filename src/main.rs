mod board;
mod fen;
mod move_generator;
mod types;
mod utility;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    let b = board::Board::new(fen);
    let moves = move_generator::generate_moves(&b);
}
