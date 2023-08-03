#[cfg(test)]
mod tests {
    use crate::{
        board::{Board, Move, MoveFlag},
        core::Piece,
    };

    const TEST_FENS: [&str; 3] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    ];

    #[test]
    fn test_board_to_fen() {
        for fen in TEST_FENS {
            let b = Board::new(fen);
            assert_eq!(b.to_fen(), fen.to_string());
        }
    }

    #[test]
    fn test_board_zobrist() {
        let default_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        // check initial hash is the same
        let mut b1 = Board::new(default_fen);
        let mut b2 = Board::new(default_fen);

        assert_eq!(b1.zobrist(), b2.zobrist());

        // make the same moves in different orders
        let white_e2e4 = Move { from: 52, to: 36, piece: Piece::Pawn, flag: MoveFlag::Quiet };
        let white_g1f3 = Move { from: 62, to: 45, piece: Piece::Knight, flag: MoveFlag::Quiet };
        
        let black_e7e5 = Move { from: 12, to: 28, piece: Piece::Pawn, flag: MoveFlag::Quiet };
        let black_b8c6 = Move { from: 1, to: 18, piece: Piece::Knight, flag: MoveFlag::Quiet };

        b1.make_move(white_e2e4);
        b1.make_move(black_e7e5);
        b1.make_move(white_g1f3);
        b1.make_move(black_b8c6);

        b2.make_move(white_g1f3);
        b2.make_move(black_b8c6);
        b2.make_move(white_e2e4);
        b2.make_move(black_e7e5);

        // check that same transpositions have the same hashes
        assert_eq!(b1.zobrist(), b2.zobrist());
    }
}
