use honeycomb::{
    board::{Board, Color, Move, Piece},
    search::{SearchContext, alpha_beta, evaluate, find_best_move},
};
use std::time::{Duration, Instant};

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

    let best_move = find_best_move(&mut board, 1, Duration::from_secs(1))
        .expect("starting position has legal moves");

    assert!(legal_moves.contains(&best_move));
}

#[test]
fn search_restores_the_board_after_exploring_moves() {
    let mut board = Board::starting_position();
    let before = board.clone();

    let _ = find_best_move(&mut board, 2, Duration::from_secs(1));

    assert_eq!(board, before);
}

#[test]
fn timed_out_search_restores_the_board() {
    let mut board = Board::starting_position();
    let before = board.clone();
    let mut context = SearchContext {
        deadline: Instant::now() - Duration::from_secs(1),
        nodes: 1_022,
    };

    assert_eq!(
        alpha_beta(&mut board, 4, i32::MIN + 1, i32::MAX, &mut context),
        None
    );
    assert_eq!(board, before);
}

#[test]
fn search_captures_an_unprotected_queen() {
    let mut board = Board::empty();
    board.set_piece(Color::White, Piece::King, 4);
    board.set_piece(Color::White, Piece::Rook, 0);
    board.set_piece(Color::Black, Piece::King, 60);
    board.set_piece(Color::Black, Piece::Queen, 56);

    let best_move = find_best_move(&mut board, 1, Duration::from_secs(1));

    assert_eq!(
        best_move,
        Some(Move {
            from: 0,
            to: 56,
            promotion: None,
        })
    );
}
