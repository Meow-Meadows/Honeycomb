use crate::board::{Board, Move, Piece};
use crate::eval::{evaluate, piece_value};
use crate::transposition::{Bound, Entry, TranspositionTable};
use std::cmp::Reverse;
use std::time::{Duration, Instant};
const CHECKMATE_SCORE: i32 = 100_000;

pub struct SearchContext {
    pub deadline: Instant,
    pub nodes: u64,
    pub qnodes: u64,
    pub tt_hits: u64,
    tt: TranspositionTable,
}
impl SearchContext {
    pub fn new(deadline: Instant) -> Self {
        Self {
            deadline,
            nodes: 0,
            qnodes: 0,
            tt_hits: 0,
            tt: TranspositionTable::new(1 << 18),
            // needs to be changed to actually calculate size based on memory
        }
    }

    pub fn with_hash_megabytes(deadline: Instant, megabytes: usize) -> Option<Self> {
        Some(Self {
            deadline,
            nodes: 0,
            qnodes: 0,
            tt_hits: 0,
            tt: TranspositionTable::try_with_megabytes(megabytes)?,
        })
    }

    pub fn set_hash_megabytes(&mut self, megabytes: usize) -> bool {
        let Some(table) = TranspositionTable::try_with_megabytes(megabytes) else {
            return false;
        };
        self.tt = table;
        true
    }

    pub fn clear_transposition_table(&mut self) {
        self.tt.clear();
    }

    pub fn hashfull(&self) -> u16 {
        self.tt.hashfull()
    }

    fn begin_search(&mut self, deadline: Instant) {
        self.deadline = deadline;
        self.nodes = 0;
        self.qnodes = 0;
        self.tt_hits = 0;
    }

    pub fn is_time_up(&mut self) -> bool {
        self.nodes.is_multiple_of(1024) && Instant::now() >= self.deadline
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SearchInfo {
    pub depth: u32,
    pub score: i32,
    pub nodes: u64,
    pub qnodes: u64,
    pub tt_hits: u64,
    pub hashfull: u16,
    pub elapsed: Duration,
}

impl SearchInfo {
    pub fn nps(&self) -> u64 {
        let seconds = self.elapsed.as_secs_f64();

        if seconds == 0.0 {
            return 0;
        }

        (self.nodes as f64 / seconds) as u64
    }
}

fn move_order_score(board: &Board, mv: Move) -> i32 {
    let us = board.side_to_move;
    let enemy = us.opposite();

    let moving_piece = board
        .piece_at(us, mv.from)
        .expect("a legal move must have a moving piece");

    let mut score = 0;

    if let Some(captured_piece) = board.piece_at(enemy, mv.to) {
        score += 10_000 + 10 * piece_value(captured_piece) - piece_value(moving_piece);
    }

    if let Some(promoted_piece) = mv.promotion {
        score += 20_000 + piece_value(promoted_piece);
    }

    score
}

fn ordered_legal_moves(board: &mut Board, preferred: Option<Move>) -> Vec<Move> {
    let mut moves = board.generate_legal_moves();

    moves.sort_unstable_by_key(|&mv| Reverse((Some(mv) == preferred, move_order_score(board, mv))));

    moves
}

fn quiescence(
    board: &mut Board,
    mut alpha: i32,
    beta: i32,
    ctx: &mut SearchContext,
    qply: u32,
) -> Option<i32> {
    ctx.nodes += 1;
    ctx.qnodes += 1;

    if ctx.is_time_up() {
        return None;
    }

    let in_check = board.in_check(board.side_to_move);
    let legal_moves = ordered_legal_moves(board, None);

    if legal_moves.is_empty() {
        return Some(if in_check { -CHECKMATE_SCORE } else { 0 });
    }

    if qply >= 128 {
        return None;
    }

    if !in_check {
        let stand_pat = evaluate(board);

        if stand_pat >= beta {
            return Some(beta);
        }

        alpha = alpha.max(stand_pat);
    }

    let us = board.side_to_move;
    let enemy = us.opposite();

    for mv in legal_moves {
        let is_capture = board.piece_at(enemy, mv.to).is_some();

        let is_en_passant = board.piece_at(us, mv.from) == Some(Piece::Pawn)
            && board.en_passant == Some(mv.to)
            && mv.from % 8 != mv.to % 8;

        let is_promotion = mv.promotion.is_some();

        if !in_check && !is_capture && !is_en_passant && !is_promotion {
            continue;
        }

        let undo = board.make_move(mv);
        let result = quiescence(board, -beta, -alpha, ctx, qply + 1);
        board.unmake_move(undo);

        let score = -result?;

        if score >= beta {
            return Some(beta);
        }

        alpha = alpha.max(score);
    }

    Some(alpha)
}

pub fn alpha_beta(
    board: &mut Board,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    ctx: &mut SearchContext,
) -> Option<i32> {
    if depth == 0 {
        return quiescence(board, alpha, beta, ctx, 0);
    }

    ctx.nodes += 1;
    if ctx.is_time_up() {
        return None;
    }

    let original_alpha = alpha;
    let key = board.hash;
    let cached = ctx.tt.get(key);

    let preferred_move = cached.and_then(|entry| entry.best_move);

    if let Some(entry) = cached
        && entry.depth as u32 >= depth
    {
        let score_is_useful = match entry.bound {
            Bound::Exact => true,
            Bound::Lower => entry.score >= beta,
            Bound::Upper => entry.score <= alpha,
        };

        if score_is_useful {
            ctx.tt_hits += 1;
            return Some(entry.score);
        }
    }

    let legal_moves = ordered_legal_moves(board, preferred_move);

    if legal_moves.is_empty() {
        return Some(if board.in_check(board.side_to_move) {
            -CHECKMATE_SCORE
        } else {
            0
        });
    }

    let mut best_move = None;
    let mut best_score = i32::MIN + 1;

    for mv in legal_moves {
        let undo = board.make_move(mv);
        let result = alpha_beta(board, depth - 1, -beta, -alpha, ctx);
        board.unmake_move(undo);

        let score = -result?;

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }

        if score > alpha {
            alpha = score;
        }

        if score >= beta {
            break;
        }
    }

    let bound = if alpha <= original_alpha {
        Bound::Upper
    } else if alpha >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };

    ctx.tt.store(Entry {
        key,
        depth: depth.min(u8::MAX as u32) as u8,
        score: alpha,
        bound,
        best_move,
    });

    Some(alpha)
}

pub fn find_best_move(board: &mut Board, depth: u32, limit: Duration) -> Option<Move> {
    find_best_move_with_info(board, depth, limit, |_| {})
}

pub fn find_best_move_with_info(
    board: &mut Board,
    depth: u32,
    limit: Duration,
    mut report: impl FnMut(SearchInfo),
) -> Option<Move> {
    let started = Instant::now();

    let mut ctx = SearchContext::new(started + limit);
    find_best_move_with_context(board, depth, limit, &mut ctx, &mut report)
}

pub fn find_best_move_with_context(
    board: &mut Board,
    depth: u32,
    limit: Duration,
    ctx: &mut SearchContext,
    mut report: impl FnMut(SearchInfo),
) -> Option<Move> {
    let started = Instant::now();
    ctx.begin_search(started + limit);

    let moves = ordered_legal_moves(board, None);
    if moves.is_empty() {
        return None;
    }

    let mut best_move = moves.first().copied();

    for curr_depth in 1..=depth {
        let mut curr_best_move = None;
        let mut curr_best_score = i32::MIN + 1;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;
        let mut completed_depth = true;

        for &mv in &moves {
            let undo = board.make_move(mv);
            let res = alpha_beta(board, curr_depth - 1, -beta, -alpha, ctx);
            board.unmake_move(undo);

            match res {
                Some(score) => {
                    if (-score) > curr_best_score {
                        curr_best_score = -score;
                        curr_best_move = Some(mv);
                    }
                    if (-score) > alpha {
                        alpha = -score;
                    }
                }
                None => {
                    completed_depth = false;
                    break;
                }
            }
        }
        if completed_depth {
            best_move = curr_best_move;

            report(SearchInfo {
                depth: curr_depth,
                score: curr_best_score,
                nodes: ctx.nodes,
                qnodes: ctx.qnodes,
                tt_hits: ctx.tt_hits,
                hashfull: ctx.hashfull(),
                elapsed: started.elapsed(),
            });
        } else {
            break;
        }
        if ctx.is_time_up() {
            break;
        }
    }

    //let mut best_score = i32::MIN + 1;
    //let mut alpha = i32::MIN + 1;
    //let beta = i32::MAX;

    best_move
}
