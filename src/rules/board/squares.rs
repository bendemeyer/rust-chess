use crate::rules::Color;
use crate::util::FxIndexSet;
use crate::util::errors::ValueError;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BoardSquare {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

pub enum BoardColumn {
    A, B, C, D, E, F, G, H,
}

pub enum BoardRow {
    Row1, Row2, Row3, Row4, Row5, Row6, Row7, Row8,
}

pub enum BoardDiagonal {
    A1H8, B1H7, C1H6, D1H5, E1H4, F1H3, G1H2,
    B1A2, C1A3, D1A4, E1A5, F1A6, G1A7, H1A8,
    A2G8, A3F8, A4E8, A5D8, A6C8, A7B8,
    H2B8, H3C8, H4D8, H5E8, H6F8, H7G8,
}


lazy_static! {
    pub static ref ROW_1: FxIndexSet<u8> = [  0u8,  1u8,  2u8,  3u8,  4u8,  5u8,  6u8,  7u8 ].into_iter().collect();
    pub static ref ROW_2: FxIndexSet<u8> = [  8u8,  9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8 ].into_iter().collect();
    pub static ref ROW_3: FxIndexSet<u8> = [ 16u8, 17u8, 18u8, 19u8, 20u8, 21u8, 22u8, 23u8 ].into_iter().collect();
    pub static ref ROW_4: FxIndexSet<u8> = [ 24u8, 25u8, 26u8, 27u8, 28u8, 29u8, 30u8, 31u8 ].into_iter().collect();
    pub static ref ROW_5: FxIndexSet<u8> = [ 32u8, 33u8, 34u8, 35u8, 36u8, 37u8, 38u8, 39u8 ].into_iter().collect();
    pub static ref ROW_6: FxIndexSet<u8> = [ 40u8, 41u8, 42u8, 43u8, 44u8, 45u8, 46u8, 47u8 ].into_iter().collect();
    pub static ref ROW_7: FxIndexSet<u8> = [ 48u8, 49u8, 50u8, 51u8, 52u8, 53u8, 54u8, 55u8 ].into_iter().collect();
    pub static ref ROW_8: FxIndexSet<u8> = [ 56u8, 57u8, 58u8, 59u8, 60u8, 61u8, 62u8, 63u8 ].into_iter().collect();
}


pub fn square_in_row(square: &u8, row: BoardRow) -> bool {
    return match row {
        BoardRow::Row1 => ROW_1.contains(square),
        BoardRow::Row2 => ROW_2.contains(square),
        BoardRow::Row3 => ROW_3.contains(square),
        BoardRow::Row4 => ROW_4.contains(square),
        BoardRow::Row5 => ROW_5.contains(square),
        BoardRow::Row6 => ROW_6.contains(square),
        BoardRow::Row7 => ROW_7.contains(square),
        BoardRow::Row8 => ROW_8.contains(square),
    }
}

pub fn move_from_square(square: u8, col_shift: i8, row_shift: i8) -> Result<u8, ValueError> {
    let start = square as i8;
    let col_position = start % 8;
    if col_position + col_shift < 0 || col_position + col_shift > 7 {
        return Err(ValueError::new("Invalid column shift provided"));
    }
    let result = start + col_shift + (row_shift * 8);
    if result < 0 || result > 63 {
        return Err(ValueError::new("Invalid row shift provided"));
    }
    return Ok(result as u8);
}

pub fn get_square_from_col_and_row(col: u8, row: u8) -> u8 {
    if col > 7 { panic!("Invalid column supplied") }
    if row > 7 { panic!("Invalid row supplied") }
    return col + (row * 8);
}

pub fn get_col_and_row_from_square(square: u8) -> [u8; 2] {
    if square > 63 { panic!("Invalid square supplied") }
    return [ square % 8, square / 8 ]
}

pub fn is_second_rank(square: u8, color: Color) -> bool {
    match color {
        Color::White => get_col_and_row_from_square(square)[1] == 1,
        Color::Black => get_col_and_row_from_square(square)[1] == 6,
    }
}

pub fn is_fourth_rank(square: u8, color: Color) -> bool {
    match color {
        Color::White => get_col_and_row_from_square(square)[1] == 3,
        Color::Black => get_col_and_row_from_square(square)[1] == 4,
    }
}

pub fn is_eighth_rank(square: u8, color: Color) -> bool {
    match color {
        Color::White => get_col_and_row_from_square(square)[1] == 7,
        Color::Black => get_col_and_row_from_square(square)[1] == 0,
    }
}

fn map_col_to_name(col: u8) -> char {
    return match col {
        0 => 'a', 1 => 'b', 2 => 'c', 3 => 'd', 4 => 'e', 5 => 'f', 6 => 'g', 7 => 'h',
        _ => panic!("Invalid column supplied")
    }
}

fn map_row_to_name(row: u8) -> char {
    return match row {
        0 => '1', 1 => '2', 2 => '3', 3 => '4', 4 => '5', 5 => '6', 6 => '7', 7 => '8',
        _ => panic!("Invalid column supplied")
    }
}

fn map_name_to_col(name: char) -> u8 {
    return match name {
        'a' => 0, 'b' => 1, 'c' => 2, 'd' => 3, 'e' => 4, 'f' => 5, 'g' => 6, 'h' => 7,
        _ => panic!("Invalid column supplied")
    }
}

fn map_name_to_row(name: char) -> u8 {
    return match name {
        '1' => 0, '2' => 1, '3' => 2, '4' => 3, '5' => 4, '6' => 5, '7' => 6, '8' => 7,
        _ => panic!("Invalid column supplied")
    }
}

fn get_col_and_row_from_notation(note: &str) -> [char; 2] {
    if note.len() != 2 { panic!("Invalid square notation!") };
    return [ note.chars().into_iter().nth(0).unwrap(), note.chars().into_iter().nth(1).unwrap() ]
}

fn get_notation_from_col_and_row(arr: [char; 2]) -> String {
    return arr.into_iter().collect()
}


pub fn get_notation_for_square(square: u8) -> Result<[char; 2], ValueError> {
    if square > 63 { return Err(ValueError::new("Cannot get notation for invalid square")) }
    let [ col, row ] = get_col_and_row_from_square(square);
    return Ok([map_col_to_name(col), map_row_to_name(row)])
}

pub fn get_square_from_notation(note: &str) -> u8 {
    let [col, row] = get_col_and_row_from_notation(note);
    return get_square_from_col_and_row(map_name_to_col(col), map_name_to_row(row));
}


impl BoardSquare {
    pub fn apply_movement(&self, col_shift: i8, row_shift: i8) -> Result<BoardSquare, ValueError> {
        return match move_from_square(self.value(), col_shift, row_shift) {
            Ok(v) => Ok(Self::from_value(v)),
            Err(e) => Err(e),
        }
    }

    pub fn value(&self) -> u8 {
        return match self {
            &BoardSquare::A1 =>  0, &BoardSquare::B1 =>  1, &BoardSquare::C1 =>  2, &BoardSquare::D1 =>  3,
            &BoardSquare::E1 =>  4, &BoardSquare::F1 =>  5, &BoardSquare::G1 =>  6, &BoardSquare::H1 =>  7,
            &BoardSquare::A2 =>  8, &BoardSquare::B2 =>  9, &BoardSquare::C2 => 10, &BoardSquare::D2 => 11,
            &BoardSquare::E2 => 12, &BoardSquare::F2 => 13, &BoardSquare::G2 => 14, &BoardSquare::H2 => 15,
            &BoardSquare::A3 => 16, &BoardSquare::B3 => 17, &BoardSquare::C3 => 18, &BoardSquare::D3 => 19,
            &BoardSquare::E3 => 20, &BoardSquare::F3 => 21, &BoardSquare::G3 => 22, &BoardSquare::H3 => 23,
            &BoardSquare::A4 => 24, &BoardSquare::B4 => 25, &BoardSquare::C4 => 26, &BoardSquare::D4 => 27,
            &BoardSquare::E4 => 28, &BoardSquare::F4 => 29, &BoardSquare::G4 => 30, &BoardSquare::H4 => 31,
            &BoardSquare::A5 => 32, &BoardSquare::B5 => 33, &BoardSquare::C5 => 34, &BoardSquare::D5 => 35,
            &BoardSquare::E5 => 36, &BoardSquare::F5 => 37, &BoardSquare::G5 => 38, &BoardSquare::H5 => 39,
            &BoardSquare::A6 => 40, &BoardSquare::B6 => 41, &BoardSquare::C6 => 42, &BoardSquare::D6 => 43,
            &BoardSquare::E6 => 44, &BoardSquare::F6 => 45, &BoardSquare::G6 => 46, &BoardSquare::H6 => 47,
            &BoardSquare::A7 => 48, &BoardSquare::B7 => 49, &BoardSquare::C7 => 50, &BoardSquare::D7 => 51,
            &BoardSquare::E7 => 52, &BoardSquare::F7 => 53, &BoardSquare::G7 => 54, &BoardSquare::H7 => 55,
            &BoardSquare::A8 => 56, &BoardSquare::B8 => 57, &BoardSquare::C8 => 58, &BoardSquare::D8 => 59,
            &BoardSquare::E8 => 60, &BoardSquare::F8 => 61, &BoardSquare::G8 => 62, &BoardSquare::H8 => 63,
        }
    }

    pub fn get_notation(&self) -> [char; 2] {
        return get_notation_for_square(self.value()).unwrap()
    }

    pub fn get_notation_string(&self) -> String {
        return self.get_notation().iter().collect();
    }

    pub fn from_value(val: u8) -> BoardSquare {
        return match val {
            0  => BoardSquare::A1,  1 => BoardSquare::B1,  2 => BoardSquare::C1,  3 => BoardSquare::D1,
            4  => BoardSquare::E1,  5 => BoardSquare::F1,  6 => BoardSquare::G1,  7 => BoardSquare::H1,
            8  => BoardSquare::A2,  9 => BoardSquare::B2, 10 => BoardSquare::C2, 11 => BoardSquare::D2,
            12 => BoardSquare::E2, 13 => BoardSquare::F2, 14 => BoardSquare::G2, 15 => BoardSquare::H2,
            16 => BoardSquare::A3, 17 => BoardSquare::B3, 18 => BoardSquare::C3, 19 => BoardSquare::D3,
            20 => BoardSquare::E3, 21 => BoardSquare::F3, 22 => BoardSquare::G3, 23 => BoardSquare::H3,
            24 => BoardSquare::A4, 25 => BoardSquare::B4, 26 => BoardSquare::C4, 27 => BoardSquare::D4,
            28 => BoardSquare::E4, 29 => BoardSquare::F4, 30 => BoardSquare::G4, 31 => BoardSquare::H4,
            32 => BoardSquare::A5, 33 => BoardSquare::B5, 34 => BoardSquare::C5, 35 => BoardSquare::D5,
            36 => BoardSquare::E5, 37 => BoardSquare::F5, 38 => BoardSquare::G5, 39 => BoardSquare::H5,
            40 => BoardSquare::A6, 41 => BoardSquare::B6, 42 => BoardSquare::C6, 43 => BoardSquare::D6,
            44 => BoardSquare::E6, 45 => BoardSquare::F6, 46 => BoardSquare::G6, 47 => BoardSquare::H6,
            48 => BoardSquare::A7, 49 => BoardSquare::B7, 50 => BoardSquare::C7, 51 => BoardSquare::D7,
            52 => BoardSquare::E7, 53 => BoardSquare::F7, 54 => BoardSquare::G7, 55 => BoardSquare::H7,
            56 => BoardSquare::A8, 57 => BoardSquare::B8, 58 => BoardSquare::C8, 59 => BoardSquare::D8,
            60 => BoardSquare::E8, 61 => BoardSquare::F8, 62 => BoardSquare::G8, 63 => BoardSquare::H8,
            _ => panic!("Not a valid square number")
        }
    }

    pub fn from_notation(note: &str) -> Self {
        return Self::from_value(get_square_from_notation(note));
    }
}