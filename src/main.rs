mod board;
mod fen;
mod move_generator;
mod types;
mod utility;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    let mut b = board::Board::new(fen);
    let mg = move_generator::MoveGenerator::new();
    let moves = mg.generate_moves(&b);
}
