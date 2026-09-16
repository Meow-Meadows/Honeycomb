use honeycomb::{
    board::{Board, Move},
    search::{SearchContext, alpha_beta, find_best_move},
};
use std::time::{Duration, Instant};

fn context() -> SearchContext {
    SearchContext {
        deadline: Instant::now() + Duration::from_secs(10),
        nodes: 0,
    }
}

#[test]
fn depth_zero_finishes_a_capture_and_restores_the_board() {
    let mut board = Board::from_fen("7k/8/8/8/8/8/q7/R6K w - - 0 1").unwrap();
    let before = board.clone();
    // Rxa2 leaves White with a rook, rather than a 400-point deficit.
    assert_eq!(
        alpha_beta(&mut board, 0, -200_000, 200_000, &mut context()),
        Some(500)
    );
    assert_eq!(board, before);
}

#[test]
fn depth_one_avoids_a_poisoned_pawn() {
    let mut board = Board::from_fen("3r2k1/8/8/3p4/8/8/8/3Q2K1 w - - 0 1").unwrap();
    let before = board.clone();
    let poisoned_capture = Move {
        from: 3,
        to: 35,
        promotion: None,
    };
    let legal = board.generate_legal_moves();
    assert!(legal.contains(&poisoned_capture));
    // Qxd5 wins a pawn only until ...Rxd5, beyond normal depth one.
    let best = find_best_move(&mut board, 1, Duration::from_secs(10)).unwrap();
    assert!(legal.contains(&best));
    assert_ne!(best, poisoned_capture);
    assert_eq!(board, before);
}

#[test]
fn checked_positions_search_quiet_escapes_instead_of_standing_pat() {
    let mut board = Board::from_fen("k3r3/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let before = board.clone();
    assert!(board.in_check(board.side_to_move));
    // Every escape is a quiet king move. A stand-pat beta cutoff would
    // return without visiting any child because -500 >= -600.
    assert!(board.generate_legal_moves().iter().all(|mv| {
        board
            .piece_at(board.side_to_move.opposite(), mv.to)
            .is_none()
    }));
    let mut ctx = context();
    assert_eq!(
        alpha_beta(&mut board, 0, -1_000, -600, &mut ctx),
        Some(-600)
    );
    assert!(ctx.nodes > 1, "check requires searching an escape");
    assert_eq!(board, before);
}

#[test]
fn depth_zero_searches_en_passant_for_both_colors() {
    for fen in [
        "4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "4k3/8/8/8/3Pp3/8/8/4K3 b - d3 0 1",
    ] {
        let mut board = Board::from_fen(fen).unwrap();
        let before = board.clone();
        assert_eq!(
            alpha_beta(&mut board, 0, -200_000, 200_000, &mut context()),
            Some(100),
            "{fen}"
        );
        assert_eq!(board, before);
    }
}

#[test]
fn depth_zero_searches_non_capture_promotions_for_both_colors() {
    for fen in [
        "7k/P7/8/8/8/8/8/7K w - - 0 1",
        "7k/8/8/8/8/8/p7/7K b - - 0 1",
    ] {
        let mut board = Board::from_fen(fen).unwrap();
        let before = board.clone();
        assert_eq!(
            alpha_beta(&mut board, 0, -200_000, 200_000, &mut context()),
            Some(900),
            "{fen}"
        );
        assert_eq!(board, before);
    }
}

#[test]
fn stand_pat_beta_cutoff_preserves_the_board() {
    let mut board = Board::from_fen("7k/8/8/8/8/8/8/R6K w - - 0 1").unwrap();
    let before = board.clone();
    assert_eq!(
        alpha_beta(&mut board, 0, -200_000, 100, &mut context()),
        Some(100)
    );
    assert_eq!(board, before);
}

#[test]
fn timeout_inside_a_capture_restores_the_board() {
    let mut board = Board::from_fen("7k/8/8/8/8/8/q7/R6K w - - 0 1").unwrap();
    let before = board.clone();
    let mut ctx = SearchContext {
        deadline: Instant::now() - Duration::from_secs(1),
        // Polling occurs every 1024 nodes: expire in the capture's child,
        // not on entry, so this exercises cancellation after make_move.
        nodes: 1_022,
    };
    assert_eq!(alpha_beta(&mut board, 0, -200_000, 200_000, &mut ctx), None);
    assert!(ctx.nodes >= 1_024);
    assert_eq!(board, before);
}

#[test]
fn search_returns_a_legal_fallback_when_no_iteration_runs() {
    let mut board = Board::starting_position();
    let before = board.clone();
    let legal = board.generate_legal_moves();
    let best = find_best_move(&mut board, 0, Duration::ZERO).unwrap();
    assert!(legal.contains(&best));
    assert_eq!(board, before);
}
