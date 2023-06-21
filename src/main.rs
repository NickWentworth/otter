mod board;
mod core;
mod move_generator;
mod search;
mod tests;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();

    let mut b = board::Board::new(fen);

    println!("{}", search::minimax(&mut b, 5));
}
