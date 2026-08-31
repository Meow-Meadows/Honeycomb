use honeycomb::board::Board;

fn empty_board() -> Board {
    Board {
        white_pawns: 0,
        white_knights: 0,
        white_bishops: 0,
        white_rooks: 0,
        white_queens: 0,
        white_king: 0,
        black_pawns: 0,
        black_knights: 0,
        black_bishops: 0,
        black_rooks: 0,
        black_queens: 0,
        black_king: 0,
        white_to_move: true,
    }
}

#[test]
fn detects_rook_check_on_open_file() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.black_rooks = 1u64 << 60; // e8

    assert!(board.in_check(true));
}

#[test]
fn blocked_rook_does_not_check_king() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.white_pawns = 1u64 << 12; // e2 blocks the file
    board.black_rooks = 1u64 << 60; // e8

    assert!(!board.in_check(true));
}

#[test]
fn detects_black_pawn_check() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.black_pawns = 1u64 << 11; // d2 attacks e1

    assert!(board.in_check(true));
}

#[test]
fn detects_white_pawn_check() {
    let mut board = empty_board();
    board.black_king = 1u64 << 60; // e8
    board.white_pawns = 1u64 << 51; // d7 attacks e8

    assert!(board.in_check(false));
}

#[test]
fn pawn_attack_does_not_wrap_across_board_edge() {
    let mut board = empty_board();
    board.black_king = 1u64 << 15; // h2
    board.white_pawns = 1u64 << 8; // a2 must not attack h2

    assert!(!board.in_check(false));
}

#[test]
fn detects_knight_check() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.black_knights = 1u64 << 21; // f3 attacks e1

    assert!(board.in_check(true));
}

#[test]
fn detects_adjacent_enemy_king() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.black_king = 1u64 << 12; // e2

    assert!(board.in_check(true));
}

#[test]
fn detects_bishop_check_on_open_diagonal() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.black_bishops = 1u64 << 31; // h4

    assert!(board.in_check(true));
}

#[test]
fn blocked_bishop_does_not_check_king() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.white_pawns = 1u64 << 13; // f2 blocks the diagonal
    board.black_bishops = 1u64 << 31; // h4

    assert!(!board.in_check(true));
}

#[test]
fn detects_queen_check_on_diagonal() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.black_queens = 1u64 << 31; // h4

    assert!(board.in_check(true));
}

#[test]
fn detects_queen_check_on_file() {
    let mut board = empty_board();
    board.white_king = 1u64 << 4; // e1
    board.black_queens = 1u64 << 60; // e8

    assert!(board.in_check(true));
}
