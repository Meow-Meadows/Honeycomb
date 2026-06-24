mod board;
use board::Board;

fn main() {
    let b = Board::starting_position();
    println!("honeycomb chess engine :3");
    println!("white pawns: {:064b}", b.white_pawns);
}
