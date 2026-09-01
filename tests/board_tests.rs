use honeycomb::board::{Board, Color, Piece};

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
