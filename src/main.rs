mod board;
mod move_generator;
mod types;

fn main() {
    let b = board::Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    b.generate_moves();
}
