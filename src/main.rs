mod board;
mod move_generator;
mod types;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    let b = board::Board::new(fen);
    let mg = move_generator::MoveGenerator::new();

    let moves = mg.generate_moves(&b);
    for m in moves.iter() {
        println!("{}", m);
    }
    println!("{}", moves.len());
}
