mod board;
mod fen;
mod move_generator;
mod types;
mod utility;

fn main() {
    let b = board::Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    b.generate_moves();
}
