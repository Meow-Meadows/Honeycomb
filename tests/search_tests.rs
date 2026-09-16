use honeycomb::{
    board::{Board, Color, Move, Piece},
    search::{SearchContext, alpha_beta, find_best_move},
};
use std::time::{Duration, Instant};

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

    let best = find_best_move(&mut board, 2, Duration::from_secs(1))
        .expect("search should complete and return a move");
    assert!(before.clone().generate_legal_moves().contains(&best));

    assert_eq!(board, before);
}

#[test]
fn timed_out_search_restores_the_board() {
    let mut board = Board::starting_position();
    let before = board.clone();
    let mut context = SearchContext {
        deadline: Instant::now() - Duration::from_secs(1),
        nodes: 0,
    };

    assert_eq!(
        alpha_beta(&mut board, 6, i32::MIN + 1, i32::MAX, &mut context),
        None
    );
    assert!(
        context.nodes > 1,
        "timeout should unwind a search in progress"
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

#[test]
fn checkmate_has_no_best_move_and_scores_as_a_loss_at_every_depth() {
    let mut board = Board::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    let before = board.clone();
    assert!(board.in_check(Color::Black));
    assert!(board.generate_legal_moves().is_empty());
    assert_eq!(find_best_move(&mut board, 2, Duration::from_secs(1)), None);
    for depth in [0, 1, 2] {
        let mut ctx = SearchContext {
            deadline: Instant::now() + Duration::from_secs(10),
            nodes: 0,
        };
        let score = alpha_beta(&mut board, depth, i32::MIN + 1, i32::MAX, &mut ctx).unwrap();
        assert!(
            score < -10_000,
            "checkmate evaluated as {score} at depth {depth}"
        );
        assert_eq!(board, before);
    }
}

#[test]
fn stalemate_has_no_best_move_and_scores_as_a_draw_at_every_depth() {
    let mut board = Board::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
    let before = board.clone();
    assert!(!board.in_check(Color::Black));
    assert!(board.generate_legal_moves().is_empty());
    assert_eq!(find_best_move(&mut board, 2, Duration::from_secs(1)), None);
    for depth in [0, 1, 2] {
        let mut ctx = SearchContext {
            deadline: Instant::now() + Duration::from_secs(10),
            nodes: 0,
        };
        assert_eq!(
            alpha_beta(&mut board, depth, i32::MIN + 1, i32::MAX, &mut ctx),
            Some(0),
            "depth {depth}"
        );
        assert_eq!(board, before);
    }
}
