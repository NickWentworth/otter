mod board;
mod core;
mod engine;
mod search;
mod tests;

fn main() {
    let fen = "8/6pk/8/4bq1P/5P2/4B1p1/Q1p5/5K2 b - - 0 1".to_string();

    let mut e = engine::Engine::new(fen);
    e.uci();
}
