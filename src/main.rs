mod board;
mod core;
mod search;
mod tests;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();

    let mut b = board::Board::new(fen);
    let mut t = search::TranspositionTable::<search::ScoreData>::new(512);

    for _ in 0..5 {
        let i = std::time::Instant::now();
        let (mov, score) = search::alpha_beta(&mut b, &mut t, 8);

        println!("{:?}", i.elapsed());
        println!("{}: {}", mov, score);
    }

    t.print_stats();
}
