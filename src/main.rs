use honeycomb::board::{Board, Color, Piece};

fn main() {
    let board = Board::starting_position();
    println!("honeycomb chess engine :3");
    println!(
        "white pawns: {:064b}",
        board.bitboard(Color::White, Piece::Pawn),
    );
}
