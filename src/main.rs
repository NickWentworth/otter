mod board;
mod core;
mod engine;
mod search;
mod tests;

fn main() {
    let mut e = engine::Engine::new();
    e.uci();
}
