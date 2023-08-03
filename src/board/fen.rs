// default FEN string to describe start of game
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// regex to match semi-valid board states, expects any digit N to be reduced to N 1's
const FEN_REGEX: &str = r"^((P|N|B|R|Q|K|p|n|b|r|q|k|1){8}/){7}(P|N|B|R|Q|K|p|n|b|r|q|k|1){8} (w|b) (-|(K?Q?k?q?)) (-|[a-h](3|6)) [[:digit:]]* [[:digit:]]*$";
// portions:               pieces                                                             turn  castling       en passant     halfmove     fullmove

/// Checks that fen is mostly legal (is in the correct format)
///
/// Certain cases such as a board full of kings would also pass, but this is a starting point
pub fn check_valid_fen(fen: &str) -> bool {
    let regex = regex::Regex::new(FEN_REGEX);

    // to make regex matching more exact, replace all occurrences of a digit with that number of 1's within piece data segment
    // this helps to ensure 8 pieces per rank and 8 ranks in total
    let first_space = fen.find(' ').unwrap_or(fen.len());
    let mut piece_data = fen.split_at(first_space).0.to_string();

    for i in 2..=8 {
        piece_data = piece_data.replace(&i.to_string(), &"1".repeat(i));
    }

    // now rebuild the expanded fen with the updated piece data
    // operation cannot be done on entire fen because numbers can occur elsewhere and shouldn't be expanded
    let mut expanded_fen = fen.to_string();
    expanded_fen.replace_range(0..first_space, &piece_data);

    match regex {
        Err(_) => false,
        Ok(valid_regex) => valid_regex.is_match(&expanded_fen),
    }
}
