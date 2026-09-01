use honeycomb::board::{Board, Color, Piece};

fn main() {
    let mut board = Board::starting_position();
    println!("honeycomb chess engine :3");
    println!(
        "white pawns: {:064b}",
        board.bitboard(Color::White, Piece::Pawn),
    );

    board.perft(6);
}
