use std::collections::{HashSet, VecDeque};

use crate::rules::Color;

use crate::rules::board::squares::BoardSquare;
use crate::rules::pieces::{PieceType, Piece};

use super::errors::InputError;


pub static STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";


pub fn get_notation_for_piece(piece: Piece) -> char {
    let c = match piece.piece_type {
        PieceType::Pawn   => 'P',
        PieceType::Knight => 'N',
        PieceType::Bishop => 'B',
        PieceType::Rook   => 'R',
        PieceType::Queen  => 'Q',
        PieceType::King   => 'K',
    };
    return if piece.color == Color::White { c } else { c.to_lowercase().next().unwrap() };
}


fn get_piece_for_notation(c: char) -> Result<Piece, InputError> {
    return match c {
        'P' => Ok(Piece { color: Color::White, piece_type: PieceType::Pawn   }), 'p' => Ok(Piece { color: Color::Black, piece_type: PieceType::Pawn   }),
        'N' => Ok(Piece { color: Color::White, piece_type: PieceType::Knight }), 'n' => Ok(Piece { color: Color::Black, piece_type: PieceType::Knight }),
        'B' => Ok(Piece { color: Color::White, piece_type: PieceType::Bishop }), 'b' => Ok(Piece { color: Color::Black, piece_type: PieceType::Bishop }),
        'R' => Ok(Piece { color: Color::White, piece_type: PieceType::Rook   }), 'r' => Ok(Piece { color: Color::Black, piece_type: PieceType::Rook   }),
        'Q' => Ok(Piece { color: Color::White, piece_type: PieceType::Queen  }), 'q' => Ok(Piece { color: Color::Black, piece_type: PieceType::Queen  }),
        'K' => Ok(Piece { color: Color::White, piece_type: PieceType::King   }), 'k' => Ok(Piece { color: Color::Black, piece_type: PieceType::King   }),
        _ => Err(InputError::new("Not a valid piece identifier")),
    }
}


fn get_notation_for_row(row: [Option<Piece>; 8]) -> String {
    let mut empty_count: u8 = 0;
    let mut row_string = String::new();
    for option in row.iter() {
        match option {
            None => empty_count += 1,
            Some(piece) => {
                if empty_count > 0 { row_string.push(empty_count.to_string().chars().nth(0).unwrap()) }
                empty_count = 0;
                row_string.push(get_notation_for_piece(*piece));
            }
        }
    }
    if empty_count > 0 { row_string.push(empty_count.to_string().chars().nth(0).unwrap()) }
    return row_string;
}


fn get_row_from_notation(fen: &str) -> [Option<Piece>; 8] {
    let mut row: [Option<Piece>; 8] = Default::default();
    let mut index: usize = 0;
    for note in fen.chars() {
        match get_piece_for_notation(note) {
            Ok(piece) => { row[index] = Some(piece); index += 1; },
            Err(_e) => {
                let empty_count = note.to_string().parse::<u8>().unwrap();
                for _ in 1..=empty_count { row[index] = None; index += 1; }
            }
        }
    }
    return row;
}


pub fn get_notation_for_board(board: [[Option<Piece>; 8]; 8]) -> String {
    return board.into_iter().map(|row| get_notation_for_row(row)).collect::<Vec<String>>().join("/");
}


fn get_board_from_notation(fen: &str) -> [[Option<Piece>; 8]; 8] {
    let mut board: [[Option<Piece>; 8]; 8] = Default::default();
    for (index, row_string) in fen.split("/").enumerate() {
        board[index] = get_row_from_notation(row_string);
    }
    return board;
}


fn get_notation_for_to_move(color: Color) -> String {
    return match color {
        Color::White => String::from("w"),
        Color::Black => String::from("b"),
    }
}


fn get_to_move_from_notation(fen: &str) -> Color {
    return match fen {
        "w" => Color::White,
        "b" => Color::Black,
        _ => panic!("Invalid FEN string!"),
    }
}


fn get_notation_for_castling(castling: &Castling) -> String {
    let pairs = [(castling.white_kingside, 'K'), (castling.white_queenside, 'Q'), (castling.black_kingside, 'k'), (castling.black_kingside, 'q')];
    return match pairs.into_iter().filter_map(|(flag, note)| match flag { true => Some(note), false => None}).collect::<String>() {
        x if x.is_empty() => String::from("-"),
        y => y,
    }
}


fn get_castling_from_notation(fen: &str) -> Castling {
    if fen.eq("-") {
        return Castling { white_kingside: false, white_queenside: false, black_kingside: false, black_queenside: false }
    }
    let chars: HashSet<char> = fen.chars().collect();
    return Castling {
        white_kingside : chars.contains(&'K'),
        white_queenside: chars.contains(&'Q'),
        black_kingside : chars.contains(&'k'),
        black_queenside: chars.contains(&'q'),
    }
}


fn get_notation_for_en_passant(square: Option<BoardSquare>) -> String {
    return match square {
        None => String::from("-"),
        Some(s) => s.get_notation().into_iter().collect(),
    }
}

fn get_en_passant_from_notation(fen: &str) -> Option<BoardSquare> {
    if fen.eq("-") { return None };
    return Some(BoardSquare::from_notation(fen));
}


pub struct Castling {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}


pub struct FenBoardState {
    pub board: [[Option<Piece>; 8]; 8],
    pub to_move: Color,
    pub castling: Castling,
    pub en_passant: Option<BoardSquare>,
    pub halfmove_timer: u8,
    pub move_number: u8,
}


impl FenBoardState {
    pub fn from_fen(fen: &str) -> Self {
        let mut fields: VecDeque<&str> = fen.split(" ").collect();
        return Self {
            board: get_board_from_notation(fields.pop_front().unwrap()),
            to_move: get_to_move_from_notation(fields.pop_front().unwrap()),
            castling: get_castling_from_notation(fields.pop_front().unwrap()),
            en_passant: get_en_passant_from_notation(fields.pop_front().unwrap()),
            halfmove_timer: fields.pop_front().unwrap().parse::<u8>().unwrap(),
            move_number: fields.pop_front().unwrap().parse::<u8>().unwrap(),
        }
    }

    pub fn to_fen(&self) -> String {
        let mut fields: Vec<String> = Vec::new();
        fields.push(get_notation_for_board(self.board));
        fields.push(get_notation_for_to_move(self.to_move));
        fields.push(get_notation_for_castling(&self.castling));
        fields.push(get_notation_for_en_passant(self.en_passant));
        fields.push(self.halfmove_timer.to_string());
        fields.push(self.move_number.to_string());
        return fields.join(" ");
    }
}