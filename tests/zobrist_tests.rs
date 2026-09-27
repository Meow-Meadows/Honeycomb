use honeycomb::board::{Board, Color, Move, Piece};

#[test]
fn constructed_and_fen_boards_have_recomputable_hashes() {
    for board in [
        Board::starting_position(),
        Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 17 42").unwrap(),
        Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 b - e3 0 1").unwrap(),
    ] {
        assert_eq!(board.hash, board.compute_hash());

        // FEN parsing and the constructor encode the same starting position.
        if board == Board::starting_position() {
            assert_eq!(board.hash, Board::starting_position().hash);
        }
    }
}

#[test]
fn hash_changes_when_position_identity_changes() {
    let base = "4k3/8/8/8/8/8/8/4K3";
    let white = Board::from_fen(&format!("{base} w - - 0 1")).unwrap();
    let black = Board::from_fen(&format!("{base} b - - 0 1")).unwrap();
    let castle = Board::from_fen(&format!("{base} w K - 0 1")).unwrap();
    let ep = Board::from_fen(&format!("{base} w - e3 0 1")).unwrap();

    assert_ne!(white.hash, black.hash);
    assert_ne!(white.hash, castle.hash);
    assert_ne!(white.hash, ep.hash);
    assert_eq!(white.hash, white.compute_hash());
    assert_eq!(black.hash, black.compute_hash());
    assert_eq!(castle.hash, castle.compute_hash());
    assert_eq!(ep.hash, ep.compute_hash());
}

#[test]
fn incremental_hash_matches_recomputation_after_special_and_regular_moves() {
    let cases = [
        // Quiet move and double pawn push (sets en passant square).
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            None,
        ),
        // Castling for either side and wing.
        ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", None),
        // En passant capture.
        ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1", None),
        // Promotion and promotion capture.
        ("r3k3/P7/8/8/8/8/8/4K3 w - - 0 1", None),
        // Capturing a rook on its home square changes castling rights.
        (
            "4k2r/8/8/8/8/8/1B6/4K3 w k - 0 1",
            Some(Move {
                from: 9,
                to: 63,
                promotion: None,
            }),
        ),
    ];

    for (fen, required_move) in cases {
        let mut board = Board::from_fen(fen).unwrap();
        let original = board.clone();
        let legal_moves = board.generate_legal_moves();
        assert!(!legal_moves.is_empty(), "fixture has no legal moves: {fen}");
        if let Some(mv) = required_move {
            assert!(legal_moves.contains(&mv), "fixture lacks {mv:?}: {fen}");
        }

        for mv in legal_moves {
            let old_hash = board.hash;
            let undo = board.make_move(mv);
            assert_eq!(board.hash, board.compute_hash(), "after {mv:?} from {fen}");
            board.unmake_move(undo);
            assert_eq!(board.hash, old_hash, "unmake {mv:?} from {fen}");
            assert_eq!(
                board.hash,
                board.compute_hash(),
                "after unmake {mv:?} from {fen}"
            );
            assert_eq!(
                board, original,
                "board state after unmake {mv:?} from {fen}"
            );
        }
    }
}

#[test]
fn make_unmake_keeps_hash_consistent_through_a_move_sequence() {
    let mut board = Board::starting_position();
    let before = board.clone();
    let moves = [
        Move {
            from: 12,
            to: 28,
            promotion: None,
        }, // e2-e4
        Move {
            from: 52,
            to: 36,
            promotion: None,
        }, // e7-e5
        Move {
            from: 6,
            to: 21,
            promotion: None,
        }, // Ng1-f3
        Move {
            from: 57,
            to: 42,
            promotion: None,
        }, // Nb8-c6
    ];
    let mut undos = Vec::new();

    for mv in moves {
        assert!(board.generate_legal_moves().contains(&mv));
        undos.push(board.make_move(mv));
        assert_eq!(board.hash, board.compute_hash());
    }

    while let Some(undo) = undos.pop() {
        board.unmake_move(undo);
        assert_eq!(board.hash, board.compute_hash());
    }
    assert_eq!(board, before);
    assert_eq!(board.side_to_move, Color::White);
    assert_eq!(board.piece_at(Color::White, 12), Some(Piece::Pawn));
}
