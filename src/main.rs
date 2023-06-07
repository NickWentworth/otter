mod board;
mod move_generator;
mod types;

fn main() {
    let fen = "8/8/8/8/Kb6/8/8/8 w - - 0 1".to_string();
    let mut b = board::Board::new(fen);
    let mg = move_generator::MoveGenerator::new();

    println!("{:#?}", mg.generate_moves(&b));
}
