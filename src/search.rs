use crate::board::{Board, Color, Move, Piece};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;
const CHECKMATE_SCORE: i32 = 100_000;

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

pub fn alpha_beta(board: &mut Board, depth: u32, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let legal_moves = board.generate_legal_moves();

    if legal_moves.is_empty() {
        if board.in_check(board.side_to_move) {
            //checkmate rahh
            return -CHECKMATE_SCORE - (depth as i32);
        } else {
            //stalemate
            return 0;
        }
    }

    for mv in legal_moves {
        let undo = board.make_move(mv);
        let score = -alpha_beta(board, depth - 1, -beta, -alpha);
        board.unmake_move(undo);

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
pub fn find_best_move(board: &mut Board, depth: u32) -> Option<Move> {
    let moves = board.generate_legal_moves();
    if moves.is_empty() {
        return None;
    }

    let mut best_move = None;
    let mut best_score = i32::MIN + 1;
    let mut alpha = i32::MIN + 1;
    let beta = i32::MAX;

    for mv in moves {
        let undo = board.make_move(mv);
        let score = -alpha_beta(board, depth - 1, -beta, -alpha);
        board.unmake_move(undo);

        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }

        if score > alpha {
            alpha = score;
        }
    }

    best_move
}
