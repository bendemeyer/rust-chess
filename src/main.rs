#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};

use interface::cli::Interface;
use rules::{board::{Board, BOARD_PIECE_MOVES}, pieces::movement::{HasMove, PieceMovement}};

use crate::rules::{Color, board::squares::BoardSquare, pieces::PieceType};

#[macro_use]
extern crate lazy_static;

mod engine;
mod game;
mod interface;
mod rules;
mod util;



fn print_legal_moves(board: Board) {
    let movements: Vec<PieceMovement> = board.get_legal_moves().into_iter().map(|m| m.get_piece_movements()).flatten().collect();
    let squares: Vec<&u8> = match board.state.to_move {
        Color::White => board.piece_locations.all_white_piece_locations.iter().collect::<BTreeSet<&u8>>().into_iter().collect(),
        Color::Black => board.piece_locations.all_black_piece_locations.iter().collect::<BTreeSet<&u8>>().into_iter().rev().collect(),
    };
    for square in squares {
        let piece = board.piece_map.get(&square);
        match piece {
            Some(p) => {
                println!(
                    "{} {} on {} {}",
                    match p.color { Color::White => "White", Color::Black => "Black" },
                    match p.piece_type {
                        PieceType::Pawn => "pawn  ",
                        PieceType::Knight => "knight",
                        PieceType::Bishop => "bishop",
                        PieceType::Rook => "rook  ",
                        PieceType::Queen => "queen ",
                        PieceType::King => "king  ",
                    },
                    BoardSquare::from_value(*square).get_notation_string(),
                    movements.iter().fold(Vec::new(), |mut squares: Vec<String>, m| {
                        if m.start_square == *square {
                            squares.push(BoardSquare::from_value(m.end_square).get_notation_string())
                        };
                        squares 
                    }).into_iter().collect::<Vec<String>>().as_move_string()
                )
            }
            None => ()
        }
    }
}

fn print_moves_for_piece_at_squares<I>(piece: PieceType, squares: I) where I: Iterator<Item=u8> {
    squares.map(|s| BoardSquare::from_value(s)).for_each(|square| {
        println!(
            "Knight on {} {}",
            square.get_notation_string(),
            BOARD_PIECE_MOVES.get(&square.value()).unwrap().get(&piece).unwrap().all_squares.iter().map(|s| {
                BoardSquare::from_value(*s).get_notation_string()
            }).collect::<Vec<String>>().as_move_string()
        )
    })
}

trait MoveStringList {
    fn as_move_string(&self) -> String;
}

impl MoveStringList for Vec<String> {
    fn as_move_string(&self) -> String {
        match self.split_last() {
            None => String::from("has no legal moves"),
            Some((l, v)) if v.is_empty() => format!("can move to {}", l),
            Some((l, v)) => format!("can move to {} or {}", v.join(", "), l)
        }
    }
}


#[derive(Default)]
struct Tester {
    recurse: HashMap<String, Tester>,
    value: u8,
}

impl Tester {
    fn add_tester(&mut self, name: &str) -> &mut Tester {
        self.recurse.insert(String::from(name), Default::default());
        return self.recurse.get_mut(name).unwrap();
    }
}


fn main() {
    let mut interface = Interface::new();
    interface.init();
}
