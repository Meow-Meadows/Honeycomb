use honeycomb::board::{Board, Color, Move, Piece, WHITE_KINGSIDE};

fn empty_board() -> Board {
    Board::empty()
}

#[test]
fn detects_rook_check_on_open_file() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::Rook, 60); // e8

    assert!(board.in_check(Color::White));
}

#[test]
fn blocked_rook_does_not_check_king() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::White, Piece::Pawn, 12); // e2 blocks the file
    board.set_piece(Color::Black, Piece::Rook, 60); // e8

    assert!(!board.in_check(Color::White));
}

#[test]
fn detects_black_pawn_check() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::Pawn, 11); // d2 attacks e1

    assert!(board.in_check(Color::White));
}

#[test]
fn detects_white_pawn_check() {
    let mut board = empty_board();
    board.set_piece(Color::Black, Piece::King, 60); // e8
    board.set_piece(Color::White, Piece::Pawn, 51); // d7 attacks e8

    assert!(board.in_check(Color::Black));
}

#[test]
fn pawn_attack_does_not_wrap_across_board_edge() {
    let mut board = empty_board();
    board.set_piece(Color::Black, Piece::King, 15); // h2
    board.set_piece(Color::White, Piece::Pawn, 8); // a2 must not attack h2

    assert!(!board.in_check(Color::Black));
}

#[test]
fn detects_knight_check() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::Knight, 21); // f3 attacks e1

    assert!(board.in_check(Color::White));
}

#[test]
fn detects_adjacent_enemy_king() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::King, 12); // e2

    assert!(board.in_check(Color::White));
}

#[test]
fn detects_bishop_check_on_open_diagonal() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::Bishop, 31); // h4

    assert!(board.in_check(Color::White));
}

#[test]
fn blocked_bishop_does_not_check_king() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::White, Piece::Pawn, 13); // f2 blocks the diagonal
    board.set_piece(Color::Black, Piece::Bishop, 31); // h4

    assert!(!board.in_check(Color::White));
}

#[test]
fn detects_queen_check_on_diagonal() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::Queen, 31); // h4

    assert!(board.in_check(Color::White));
}

#[test]
fn detects_queen_check_on_file() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::Queen, 60); // e8

    assert!(board.in_check(Color::White));
}

#[test]
fn starting_position_has_20_legal_moves() {
    let mut board = Board::starting_position();

    assert_eq!(board.generate_legal_moves().len(), 20);
}

#[test]
fn starting_position_perft_depth_two_is_400() {
    let mut board = Board::starting_position();

    assert_eq!(board.perft(2), 400);
}

#[test]
fn starting_position_perft_depth_three_is_8902() {
    let mut board = Board::starting_position();

    assert_eq!(board.perft(3), 8_902);
}

#[test]
fn starting_position_perft_depth_four_is_197281() {
    let mut board = Board::starting_position();

    assert_eq!(board.perft(4), 197_281);
}

#[test]
fn starting_position_perft_depth_five_is_4_865_609() {
    let mut board = Board::starting_position();

    assert_eq!(board.perft(5), 4_865_609);
}

#[test]
fn starting_position_perft_depth_six_is_119_060_324() {
    let mut board = Board::starting_position();

    assert_eq!(board.perft(6), 119_060_324);
}

#[test]
fn make_and_unmake_restore_starting_position() {
    let mut board = Board::starting_position();
    let before = board.clone();

    let mv = board
        .generate_legal_moves()
        .into_iter()
        .find(|mv| mv.from == 12 && mv.to == 28) // e2 -> e4
        .expect("e2e4 must be legal");

    let undo = board.make_move(mv);
    board.unmake_move(undo);

    assert_eq!(board, before);
}

#[test]
fn kingside_castling_moves_king_and_rook() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::White, Piece::Rook, 7); // h1
    board.set_piece(Color::Black, Piece::King, 60); // e8
    board.castling_rights = WHITE_KINGSIDE;

    let castle = Move {
        from: 4,
        to: 6,
        promotion: None,
    };

    assert!(board.generate_legal_moves().contains(&castle));

    board.make_move(castle);

    assert_eq!(board.bitboard(Color::White, Piece::King), 1u64 << 6);
    assert_eq!(board.bitboard(Color::White, Piece::Rook), 1u64 << 5);
}

#[test]
fn cannot_castle_through_an_attacked_square() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::White, Piece::Rook, 7); // h1
    board.set_piece(Color::Black, Piece::King, 60); // e8
    board.set_piece(Color::Black, Piece::Rook, 61); // f8 attacks f1
    board.castling_rights = WHITE_KINGSIDE;

    let castle = Move {
        from: 4,
        to: 6,
        promotion: None,
    };

    assert!(!board.generate_legal_moves().contains(&castle));
}

#[test]
fn en_passant_removes_the_captured_pawn() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::King, 60); // e8
    board.set_piece(Color::White, Piece::Pawn, 36); // e5
    board.set_piece(Color::Black, Piece::Pawn, 35); // d5
    board.en_passant = Some(43); // d6

    board.make_move(Move {
        from: 36,
        to: 43,
        promotion: None,
    });

    assert_eq!(board.bitboard(Color::White, Piece::Pawn), 1u64 << 43);
    assert_eq!(board.bitboard(Color::Black, Piece::Pawn), 0);
}

#[test]
fn promotion_replaces_pawn_with_selected_piece() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::King, 56); // a8
    board.set_piece(Color::White, Piece::Pawn, 52); // e7

    board.make_move(Move {
        from: 52,
        to: 60,
        promotion: Some(Piece::Queen),
    });

    assert_eq!(board.bitboard(Color::White, Piece::Pawn), 0);
    assert_eq!(board.bitboard(Color::White, Piece::Queen), 1u64 << 60);
}

#[test]
fn unmake_restores_en_passant_capture() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::King, 60); // e8
    board.set_piece(Color::White, Piece::Pawn, 36); // e5
    board.set_piece(Color::Black, Piece::Pawn, 35); // d5
    board.en_passant = Some(43); // d6
    let before = board.clone();

    let undo = board.make_move(Move {
        from: 36,
        to: 43,
        promotion: None,
    });
    board.unmake_move(undo);

    assert_eq!(board, before);
}

#[test]
fn unmake_restores_castling() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::White, Piece::Rook, 7); // h1
    board.set_piece(Color::Black, Piece::King, 60); // e8
    board.castling_rights = WHITE_KINGSIDE;
    let before = board.clone();

    let undo = board.make_move(Move {
        from: 4,
        to: 6,
        promotion: None,
    });
    board.unmake_move(undo);

    assert_eq!(board, before);
}

#[test]
fn unmake_restores_promotion() {
    let mut board = empty_board();
    board.set_piece(Color::White, Piece::King, 4); // e1
    board.set_piece(Color::Black, Piece::King, 56); // a8
    board.set_piece(Color::White, Piece::Pawn, 52); // e7
    let before = board.clone();

    let undo = board.make_move(Move {
        from: 52,
        to: 60,
        promotion: Some(Piece::Queen),
    });
    board.unmake_move(undo);

    assert_eq!(board, before);
}

#[test]
fn fen_starting_position_matches_constructor() {
    assert_eq!(
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        Some(Board::starting_position())
    );
}

#[test]
fn fen_preserves_position_and_metadata() {
    let mut expected = Board::empty();
    expected.set_piece(Color::White, Piece::King, 4);
    expected.set_piece(Color::Black, Piece::King, 60);
    expected.set_piece(Color::White, Piece::Pawn, 28);
    expected.side_to_move = Color::Black;
    expected.en_passant = Some(20);
    expected.halfmove_clock = 7;
    expected.fullmove_number = 23;
    assert_eq!(
        Board::from_fen("4k3/8/8/8/4P3/8/8/4K3 b - e3 7 23"),
        Some(expected)
    );
}

#[test]
fn fen_accepts_omitted_clocks_with_defaults() {
    let board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - -").unwrap();
    assert_eq!(board.halfmove_clock, 0);
    assert_eq!(board.fullmove_number, 1);
}

#[test]
fn fen_rejects_malformed_fields() {
    for fen in [
        "",
        "8/8/8/8/8/8/8/8 w -",
        "8/8/8/8/8/8/8/X7 w - - 0 1",
        "8/8/8/8/8/8/8/8 x - - 0 1",
        "8/8/8/8/8/8/8/8 w A - 0 1",
        "8/8/8/8/8/8/8/8 w - i3 0 1",
        "8/8/8/8/8/8/8/8 w - e4 0 1",
        "8/8/8/8/8/8/8/8 w - - nope 1",
        "8/8/8/8/8/8/8/8 w - - 0 -1",
    ] {
        assert!(Board::from_fen(fen).is_none(), "accepted {fen:?}");
    }
}

#[test]
fn fen_rejects_incorrect_rank_widths_and_counts() {
    for placement in [
        "7/8/8/8/8/8/8/8",
        "88/8/8/8/8/8/8/8",
        "8/8/8/8/8/8/8",
        "8/8/8/8/8/8/8/8/8",
        "8/8/8/8/8/8/8/7",
    ] {
        assert!(
            Board::from_fen(&format!("{placement} w - - 0 1")).is_none(),
            "accepted {placement}"
        );
    }
}

#[test]
fn generates_and_restores_castling_for_both_sides_and_wings() {
    for (side, color, from, to, rook_from, rook_to) in [
        ("w", Color::White, 4, 6, 7, 5),
        ("w", Color::White, 4, 2, 0, 3),
        ("b", Color::Black, 60, 62, 63, 61),
        ("b", Color::Black, 60, 58, 56, 59),
    ] {
        let mut board =
            Board::from_fen(&format!("r3k2r/8/8/8/8/8/8/R3K2R {side} KQkq - 0 1")).unwrap();
        let before = board.clone();
        let mv = Move {
            from,
            to,
            promotion: None,
        };
        assert!(board.generate_legal_moves().contains(&mv), "missing {mv:?}");
        let undo = board.make_move(mv);
        assert_eq!(board.piece_at(color, to), Some(Piece::King));
        assert_eq!(board.piece_at(color, rook_to), Some(Piece::Rook));
        assert_eq!(board.piece_at(color, rook_from), None);
        board.unmake_move(undo);
        assert_eq!(board, before);
    }
}

#[test]
fn generates_all_four_promotions_for_both_colors() {
    for (fen, from, to) in [
        ("k7/4P3/8/8/8/8/8/4K3 w - - 0 1", 52, 60),
        ("4k3/8/8/8/8/8/4p3/K7 b - - 0 1", 12, 4),
    ] {
        let mut board = Board::from_fen(fen).unwrap();
        let moves: Vec<_> = board
            .generate_legal_moves()
            .into_iter()
            .filter(|mv| mv.from == from && mv.to == to)
            .collect();
        assert_eq!(moves.len(), 4);
        for piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
            assert!(moves.contains(&Move {
                from,
                to,
                promotion: Some(piece)
            }));
        }
    }
}

#[test]
fn generates_legal_en_passant_for_both_colors() {
    for (fen, from, to, captured) in [
        ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1", 36, 43, 35),
        ("4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 1", 28, 19, 27),
    ] {
        let mut board = Board::from_fen(fen).unwrap();
        let before = board.clone();
        let side = board.side_to_move;
        let mv = Move {
            from,
            to,
            promotion: None,
        };
        assert!(board.generate_legal_moves().contains(&mv));
        let undo = board.make_move(mv);
        assert_eq!(board.piece_at(side, to), Some(Piece::Pawn));
        assert_eq!(board.piece_at(side.opposite(), captured), None);
        board.unmake_move(undo);
        assert_eq!(board, before);
    }
}

#[test]
fn en_passant_cannot_expose_own_king_to_rook() {
    let mut board = Board::from_fen("k7/8/8/r4pPK/8/8/8/8 w - f6 0 1").unwrap();
    let before = board.clone();
    assert!(!board.generate_legal_moves().contains(&Move {
        from: 38,
        to: 45,
        promotion: None
    }));
    assert_eq!(board, before);
}
