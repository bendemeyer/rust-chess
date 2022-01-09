use crate::rules::{board::{Board, bitboards::BitboardSquares}, Color};


pub fn evaluate_board(board: &Board) -> i16 {
    return BitboardSquares::from_board(board.position.get_all_piece_locations(Color::White) | 
        board.position.get_all_piece_locations(Color::Black)).fold(0i16, |score, s| {
            score + board.position.piece_at(&s).unwrap().material_score()
        });
}