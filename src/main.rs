mod board;
mod core;
mod search;
mod tests;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();

    let mut b = board::Board::new(fen);

    let i = std::time::Instant::now();
    let (mov, score) = search::alpha_beta(&mut b, 5);
    println!("{:?}", i.elapsed());

    println!("{}: {}", mov, score);
}
