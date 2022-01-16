pub mod board;
pub mod pieces;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn iter() -> ColorIterator {
        return ColorIterator::new();
    }

    pub fn swap(&self) -> Color {
        return match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn value(&self) -> &str {
        return match self {
            &Color::White => "white",
            &Color::Black => "black",
        }
    }
}

impl Default for Color {
    fn default() -> Self { Color::White }
}


pub struct ColorIterator {
    state: Option<Color>,
}

impl ColorIterator {
    pub fn new() -> Self {
        return Self { state: None }
    }
}

impl Iterator for ColorIterator {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.state {
            None => Some(Color::White),
            Some(Color::White) => Some(Color::Black),
            Some(Color::Black) => None
        };
        self.state = next;
        return next;
    }
}