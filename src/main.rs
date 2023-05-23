mod board;
mod fen;
mod move_generator;
mod types;
mod utility;

fn main() {
    let b = board::Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let moves = move_generator::generate_moves(&b);
}
