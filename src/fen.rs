// default FEN string to describe start of game
pub const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// regex to match semi-valid board states, doesn't check certain things (like valid number of pieces)
const FEN_REGEX: &str = r"^((P|N|B|R|Q|K|p|n|b|r|q|k|[0-8])*/?){8} (w|b) (-|(K?Q?k?q?)) (-|[a-h][1-8]) [[:digit:]]* [[:digit:]]*$";
// portions:                pieces                                 turn  castling       en passant     halfmove     fullmove

pub fn check_valid_fen(fen: &str) -> bool {
    use regex::Regex;

    let regex = Regex::new(FEN_REGEX);

    match regex {
        Err(_) => false,
        Ok(valid_regex) => valid_regex.is_match(fen),
    }
}
