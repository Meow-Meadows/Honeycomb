use honeycomb::{
    board::{Board, Color, Piece},
    eval::evaluate,
};

#[test]
fn evaluation_is_from_the_side_to_move_perspective() {
    let mut board = Board::empty();
    board.set_piece(Color::White, Piece::Queen, 3);

    assert_eq!(evaluate(&board), 900);

    board.side_to_move = Color::Black;
    assert_eq!(evaluate(&board), -900);
}
