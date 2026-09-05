use crate::board::{Board, Color, Move, Piece};
use std::time::{Duration, Instant};
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const CHECKMATE_SCORE: i32 = 100_000;

pub struct search_context {
    pub deadline: Instant,
    pub nodes: u64,
}
impl search_context {
    pub fn is_time_up(&mut self) -> bool {
        if self.nodes % 1024 == 0 && Instant::now() >= self.deadline {
            true
        }
        else {
            false
        }
    }
}

pub fn evaluate(board: &Board) -> i32 {
    let mut white_material = 0;
    let mut black_material = 0;

    for (piece, value) in [
        (Piece::Pawn, PAWN_VALUE),
        (Piece::Knight, KNIGHT_VALUE),
        (Piece::Bishop, BISHOP_VALUE),
        (Piece::Rook, ROOK_VALUE),
        (Piece::Queen, QUEEN_VALUE),
    ] {
        white_material += board.bitboard(Color::White, piece).count_ones() as i32 * value;
        black_material += board.bitboard(Color::Black, piece).count_ones() as i32 * value;
    }

    let raw_eval = white_material - black_material;

    match board.side_to_move {
        Color::White => raw_eval,
        Color::Black => -raw_eval,
    }
}

pub fn alpha_beta(board: &mut Board, depth: u32, mut alpha: i32, beta: i32, ctx: &mut search_context) -> Option<i32> {

    ctx.nodes += 1;
    if ctx.is_time_up() {
        return None;
    }

    if depth == 0 {
        return Some(evaluate(board));
    }

    let legal_moves = board.generate_legal_moves();

    if legal_moves.is_empty() {
        if board.in_check(board.side_to_move) {
            //checkmate rahh
            return Some(-CHECKMATE_SCORE - (depth as i32));
        } else {
            //stalemate
            return Some(0);
        }
    }

    for mv in legal_moves {
        let undo = board.make_move(mv);
        let res = alpha_beta(board, depth - 1, -beta, -alpha, ctx);
        board.unmake_move(undo);
        let score = -res?;

        if score >= beta {
            return Some(beta);
        }

        if score > alpha {
            alpha = score;
        }
    }

    Some(alpha)
}
pub fn find_best_move(board: &mut Board, depth: u32, limit: Duration) -> Option<Move> {
    let moves = board.generate_legal_moves();
    if moves.is_empty() {
        return None;
    }

    let mut ctx = search_context {
        deadline: Instant::now() + limit,
        nodes: 0,
    };

    let mut best_move = None;

    for curr_depth in 1..=depth {
        let mut curr_best_move = None;
        let mut curr_best_score = i32::MIN + 1;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;
        let mut completed_depth = true;

        for &mv in &moves {
            let undo = board.make_move(mv);
            let res = alpha_beta(board, curr_depth - 1, -beta, -alpha, &mut ctx);
            board.unmake_move(undo);

            match res {
                Some(score) => {
                    if (-score) > curr_best_score {
                        curr_best_score = (-score);
                        curr_best_move = Some(mv);
                    }
                    if (-score) > alpha {
                        alpha = (-score);
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
        }
        else {
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
