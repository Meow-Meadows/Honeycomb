use honeycomb::{
    board::Board,
    search::{SearchInfo, find_best_move, find_best_move_with_info},
};
use std::time::Duration;

#[test]
fn reports_completed_depths_with_cumulative_counts_and_restores_board() {
    let mut board = Board::starting_position();
    let before = board.clone();
    let mut reports = Vec::new();
    let best = find_best_move_with_info(&mut board, 3, Duration::from_secs(10), |info| {
        reports.push(info);
    });
    assert_eq!(board, before);
    assert_eq!(
        reports.iter().map(|info| info.depth).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    for info in &reports {
        assert!(info.qnodes > 0);
        assert!(info.qnodes <= info.nodes);
    }
    // Depth one enters quiescence directly for every root move.
    assert_eq!(reports[0].nodes, reports[0].qnodes);
    assert!(reports[1].nodes > reports[1].qnodes);
    for pair in reports.windows(2) {
        assert!(pair[1].nodes > pair[0].nodes);
        assert!(pair[1].qnodes > pair[0].qnodes);
        assert!(pair[1].elapsed >= pair[0].elapsed);
    }
    assert_eq!(best, find_best_move(&mut board, 3, Duration::from_secs(10)));
}

#[test]
fn reports_score_from_the_root_side_to_move() {
    let mut board = Board::from_fen("7k/8/8/8/8/8/q7/R6K w - - 0 1").unwrap();
    let mut reports = Vec::new();
    find_best_move_with_info(&mut board, 1, Duration::from_secs(10), |info| {
        reports.push(info)
    });
    // Rxa2 leaves White up a rook.
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].score, 500);
}

#[test]
fn does_not_report_a_completed_depth_when_no_iteration_runs() {
    let mut board = Board::starting_position();
    let legal = board.generate_legal_moves();
    let mut reports = Vec::new();
    let best =
        find_best_move_with_info(&mut board, 0, Duration::ZERO, |info| reports.push(info)).unwrap();
    assert!(legal.contains(&best));
    assert!(reports.is_empty());
}

#[test]
fn nps_handles_zero_and_submillisecond_durations() {
    let mut info = SearchInfo {
        depth: 1,
        score: 0,
        nodes: 100,
        qnodes: 50,
        elapsed: Duration::ZERO,
    };
    assert_eq!(info.nps(), 0);
    info.elapsed = Duration::from_micros(500);
    assert_eq!(info.nps(), 200_000);
    info.elapsed = Duration::from_secs(2);
    assert_eq!(info.nps(), 50);
}
