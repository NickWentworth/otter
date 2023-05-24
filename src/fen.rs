// default FEN string to describe start of game
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// regex to match semi-valid board states, expects any digit N to be reduced to N 1's
const FEN_REGEX: &str = r"^((P|N|B|R|Q|K|p|n|b|r|q|k|1){8}/){7}(P|N|B|R|Q|K|p|n|b|r|q|k|1){8} (w|b) (-|(K?Q?k?q?)) (-|[a-h](3|6)) [[:digit:]]* [[:digit:]]*$";
// portions:               pieces                                                             turn  castling       en passant     halfmove     fullmove

/// Checks that fen is mostly legal (is in the correct format)
///
/// Certain cases such as a board full of kings would also pass, but this is a starting point
pub fn check_valid_fen(fen: &String) -> bool {
    let regex = regex::Regex::new(FEN_REGEX);

    // to make regex matching more exact, replace all occurrences of a digit with that number of 1's
    // this helps to ensure 8 pieces per rank and 8 ranks in total
    let mut expanded_fen = fen.clone().to_string();
    for i in 2..=8 {
        expanded_fen = expanded_fen.replace(&i.to_string(), &"1".repeat(i));
    }

    match regex {
        Err(_) => false,
        Ok(valid_regex) => valid_regex.is_match(&expanded_fen),
    }
}
