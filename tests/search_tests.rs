use honeycomb::{
    board::{Board, Color, Piece},
    search::{evaluate, find_best_move},
};

#[test]
fn evaluation_is_from_the_side_to_move_perspective() {
    let mut board = Board::empty();
    board.set_piece(Color::White, Piece::Queen, 3);

    assert_eq!(evaluate(&board), 900);

    board.side_to_move = Color::Black;
    assert_eq!(evaluate(&board), -900);
}

#[test]
fn search_returns_a_legal_move() {
    let mut board = Board::starting_position();
    let legal_moves = board.generate_legal_moves();

    let best_move = find_best_move(&mut board, 1).expect("starting position has legal moves");

    assert!(legal_moves.contains(&best_move));
}

#[test]
fn search_restores_the_board_after_exploring_moves() {
    let mut board = Board::starting_position();
    let before = board.clone();

    let _ = find_best_move(&mut board, 2);

    assert_eq!(board, before);
}
