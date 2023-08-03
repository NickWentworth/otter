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

#[cfg(test)]
mod tests {
    use super::{check_valid_fen as check, *};

    #[test]
    fn test_valid_fens() {
        // all valid fen strings from various points of a game
        assert!(check(DEFAULT_FEN));
        assert!(check("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1"));
        assert!(check("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq d6 0 2"));
        assert!(check("r2q1rk1/ppp2ppp/2b1pn2/3pN3/3P4/2P1P1P1/PP3PP1/RN1QK2R b KQ - 1 10"));
        assert!(check("1r3rk1/p1p2qp1/2p1pp2/3p3Q/3P3R/2P1P1P1/PP3PP1/2KR4 w - - 7 19"));
    }

    #[test]
    fn test_invalid_fens() {
        // piece data formatting problems
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPPP/RNBQKBNR w KQkq - 0 1")); // too many pieces in a rank
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPP/RNBQKBNR w KQkq - 0 1")); // too few pieces in a rank
        assert!(!check("rnbqkbnr/pppppppX/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")); // invalid piece symbol
        
        // other data formatting problems
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR - KQkq - 0 1")); // invalid current turn
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kqKQ - 0 1")); // invalid castling order
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KkQq - 0 1")); // invalid castling order
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq a5 0 1")); // invalid en passant square
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - -1 1")); // negative halfmove
        assert!(!check("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 -1")); // negative fullmove
    }
}
