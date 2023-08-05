mod board;
mod core;
mod engine;
mod search;

fn main() {
    let mut e = engine::Engine::new();
    e.uci();
}
