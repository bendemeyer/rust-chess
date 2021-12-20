use std::collections::HashSet;

use crate::{game::Game, interface::{arguments::ParsedArgs, shell::InteractiveShell}, rules::{board::squares::BoardSquare, pieces::{PieceType, movement::Move}}};

use super::arguments::{ArgumentParser, Arguments};


fn build_argument_parser() -> ArgumentParser {
    let mut builder = ArgumentParser::builder();
    let new_command = builder.add_subcommand("new").unwrap();
    new_command.add_positional_arg("depth", true, false).unwrap();
    let list_command = builder.add_subcommand("list").unwrap();
    list_command.add_positional_arg("type", true, false).unwrap();
    let suggest_command = builder.add_subcommand("suggest").unwrap();
    suggest_command.add_positional_arg("count", false, false).unwrap();
    let _move_command = builder.add_subcommand("move").unwrap();
    let _size_command = builder.add_subcommand("size").unwrap();
    return builder.build();
}


fn format_move_elements(color: &str, piece: &str, start: &str, movement: &str, end: &str, additional: &str) -> String {
    return format!("{} {} on {} {} {}{}", color, piece, start, movement, end, additional);
}





pub struct Interface {
    shell: InteractiveShell,
    game: Game,
    confirmations: HashSet<String>,
}

impl Interface {
    pub fn new() -> Interface {
        let prompt = "chess > ";
        
        return Interface {
            shell: InteractiveShell::new(Some(prompt), build_argument_parser()),
            game: Game::new(1),
            confirmations: HashSet::from([String::from("y"), String::from("yes")]),
        }
    }

    pub fn init(&mut self) {
        loop {
            let result = self.shell.get_command();
            match result {
                Err(e) => {
                    println!("{}", e.msg);
                    break;
                },
                Ok(args) => match args {
                    ParsedArgs::SubCommand(s) => match s.name.as_str() {
                        "new"     => self.do_new(*s.args),
                        "list"    => self.do_list(*s.args),
                        "move"    => self.do_move(*s.args),
                        "size"    => self.do_size(*s.args),
                        "suggest" => self.do_suggest(*s.args),
                        x => println!("Unknown subcommand {} encountered", x)
                    },
                    ParsedArgs::Arguments(a) => {
                        self.do_default(a)
                    }
                }
            };
            println!();
        }
    }

    fn do_default(&self, args: Arguments) {

    }
    
    fn do_new(&mut self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'new' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                let confirm = self.shell.input("Are you sure you want to start a new game? All progress on the current game will be lost. (y/N): ");
                if self.confirmations.contains(&confirm.to_lowercase()) {
                    self.game = Game::new(a.args.get("depth").unwrap().parse().unwrap());
                    self.shell.output("New game started!");
                } else {
                    self.shell.output("OK, aborting...");
                }
                self.shell.empty_line();
            }
        }
    }
    
    fn do_list(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'list' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                match a.args.get("type").unwrap().as_str() {
                    "moves" => {
                        for m in self.game.get_legal_moves() {
                            self.shell.output(&self.get_text_for_move(m));
                        }
                    },
                    x => self.shell.output(&format!("Unrecognized list type: '{}'", x))
                }
            }
        }
    }
    
    fn do_move(&mut self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'move' should not have its own subcommands"),
            ParsedArgs::Arguments(_a) => {
                let start_square = self.shell.input("Which square do you want to move the piece from? ");
                let end_square = self.shell.input("Which square should it move to? ");
                let start = BoardSquare::from_notation(&start_square).value();
                let end = BoardSquare::from_notation(&end_square).value();
                let chosen_move = self.find_move(start, end);
                match chosen_move {
                    None => self.shell.output("No matching legal move found!"),
                    Some(m) => {
                        self.shell.output("Is this the move you want to make:");
                        self.shell.output(&self.get_text_for_move(&m));
                        let confirm = self.shell.input("(y/N) ");
                        match self.confirmations.contains(&confirm.to_lowercase()) {
                            false => self.shell.output("OK, aborting..."),
                            true => {
                                self.game.make_move(&m);
                                self.shell.output("Move made!")
                            }
                        }
                    }
                }
                self.shell.empty_line();
            }
        }
    }

    fn find_move(&self, start: u8, end: u8) -> Option<Move> {
        let mut chosen_move: Option<Move> = None;
        let mut promotion_type: Option<PieceType> = None;
        for m in self.game.get_legal_moves() {
            match m {
                Move::BasicMove(b) => if b.start == start && b.end == end { chosen_move = Some(m.clone()); break; },
                Move::EnPassant(e) => if e.basic_move.start == start && e.basic_move.end == end { chosen_move = Some(m.clone()); break; },
                Move::TwoSquarePawnMove(t) => if t.basic_move.start == start && t.basic_move.end == end { chosen_move = Some(m.clone());  break; },
                Move::Castle(c) => if c.king_start == start && c.king_end == end { chosen_move = Some(m.clone()); break; },
                Move::Promotion(p) => {
                    match promotion_type {
                        None => {
                            if p.basic_move.start == start && p.basic_move.end == end {
                                promotion_type = Some(self.get_promotion_choice());
                            }
                            if promotion_type == Some(p.promote_to) { chosen_move = Some(m.clone()); break; }
                        },
                        Some(t) => {
                            if t == p.promote_to { chosen_move = Some(m.clone()); break; }
                        }
                    }
                },
                Move::NewGame(_) => ()
            }
        }
        return chosen_move;
    }
    
    fn do_size(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'size' should not have its own subcommands"),
            ParsedArgs::Arguments(_a) => {
                self.shell.output(&format!("The current game engine has loaded {} positions", self.game.get_engine_size()))
            }
        }
    }
    
    fn do_suggest(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'suggest' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                
            }
        }
    }

    fn get_promotion_choice(&self) -> PieceType {
        self.shell.output("What should it promote to?");
        self.shell.output("    1. Queen");
        self.shell.output("    2. Rook");
        self.shell.output("    3. Bishop");
        self.shell.output("    4. Knight");
        let choice = self.shell.input("? ");
        match choice.as_str() {
            "1" => return PieceType::Queen,
            "2" => return PieceType::Rook,
            "3" => return PieceType::Bishop,
            "4" => return PieceType::Knight,
            _   => {
                self.shell.output("Invalid selection! Please enter either 1, 2, 3, or 4");
                return self.get_promotion_choice();
            }
        }
    }

    fn get_text_for_move(&self, m: &Move) -> String {
        match m {
            Move::BasicMove(b) => {
                let start_piece = self.game.get_piece_at(b.start).unwrap();
                let end_piece = self.game.get_piece_at(b.end);
                let movement = match end_piece {
                    Some(p) => format!("captures {} {} on", p.color.value(), p.piece_type.value()),
                    None => String::from("moves to"),
                };
                format_move_elements(
                    start_piece.color.value(),
                    start_piece.piece_type.value(),
                    &BoardSquare::from_value(b.start).get_notation_string(),
                    &movement,
                    &BoardSquare::from_value(b.end).get_notation_string(),
                    ""
                )
            },
            Move::EnPassant(e) => {
                let start_piece = self.game.get_piece_at(e.basic_move.start).unwrap();
                let end_piece = self.game.get_piece_at(e.capture_square).unwrap();
                let movement = format!("captures {} {} on {} en passant, moving to", end_piece.color.value(),
                                              end_piece.piece_type.value(), &BoardSquare::from_value(e.capture_square).get_notation_string());
                format_move_elements(
                    start_piece.color.value(),
                    start_piece.piece_type.value(),
                    &BoardSquare::from_value(e.basic_move.start).get_notation_string(),
                    &movement,
                    &BoardSquare::from_value(e.basic_move.end).get_notation_string(),
                    ""
                )
            },
            Move::Promotion(p) => {
                let start_piece = self.game.get_piece_at(p.basic_move.start).unwrap();
                let end_piece = self.game.get_piece_at(p.basic_move.end);
                let movement = match end_piece {
                    Some(p) => format!("captures {} {} on", p.color.value(), p.piece_type.value()),
                    None => String::from("moves to"),
                };
                format_move_elements(
                    start_piece.color.value(),
                    start_piece.piece_type.value(),
                    &BoardSquare::from_value(p.basic_move.start).get_notation_string(),
                    &movement,
                    &BoardSquare::from_value(p.basic_move.end).get_notation_string(),
                    &format!(" and promotes to a {}", p.promote_to.value())
                )
            },
            Move::TwoSquarePawnMove(t) => {
                let start_piece = self.game.get_piece_at(t.basic_move.start).unwrap();
                format_move_elements(
                    start_piece.color.value(),
                    start_piece.piece_type.value(),
                    &BoardSquare::from_value(t.basic_move.start).get_notation_string(),
                    "moves to",
                    &BoardSquare::from_value(t.basic_move.end).get_notation_string(),
                    ""
                )
            },
            Move::Castle(c) => {
                format!("{} castles {}", self.game.get_current_turn().value(), c.side.value())
            },
            Move::NewGame(_) => {
                String::from("new game")
            }
        }
    }
}
