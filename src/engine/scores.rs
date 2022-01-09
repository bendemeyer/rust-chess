use crate::rules::Color;


pub fn best_score(color: Color) -> i16 {
    return match color {
        Color::White => i16::MAX,
        Color::Black => i16::MIN,
    }
}


pub fn is_better(new: i16, old: i16, color: Color) -> bool {
    return match color {
        Color::White => new > old,
        Color::Black => new < old,
    }
}