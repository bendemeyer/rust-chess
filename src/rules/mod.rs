pub mod board;
pub mod pieces;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn swap(&self) -> Color {
        return match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Default for Color {
    fn default() -> Self { Color::White }
}