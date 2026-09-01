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
