use crate::rules::board::Board;

pub fn evaluate_board(board: &Board) -> i16 {
    return board.get_piece_squares().iter().fold(0i16, |score, (_square, piece)| {
        score + piece.material_score()
    })
}