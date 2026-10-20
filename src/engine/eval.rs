use crate::board::{Board, Color, Piece};

const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const BISHOP_VALUE: i32 = 330;
const ROOK_VALUE: i32 = 500;
const QUEEN_VALUE: i32 = 900;

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

pub fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => PAWN_VALUE,
        Piece::Knight => KNIGHT_VALUE,
        Piece::Bishop => BISHOP_VALUE,
        Piece::Rook => ROOK_VALUE,
        Piece::Queen => QUEEN_VALUE,
        Piece::King => 20_000,
    }
}
