mod board;
mod move_generator;
mod types;

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
    let mut b = board::Board::new(fen);
    let mg = move_generator::MoveGenerator::new();

    println!("{}", b);

    b.make_move(&mg.generate_moves(&b)[0]);
    println!("{}", b);

    b.make_move(&mg.generate_moves(&b)[0]);
    println!("{}", b);
}
