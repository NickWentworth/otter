mod board;
mod core;
mod search;
mod tests;
mod engine;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    
    let mut e = engine::Engine::new(fen);
    e.play();
}
