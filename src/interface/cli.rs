use std::collections::HashSet;

use tabled::{Table, Style};

use crate::{game::Game, interface::{arguments::ParsedArgs, shell::InteractiveShell}, rules::{board::{squares::{BoardSquare, get_notation_string_for_square}}, pieces::{PieceType, movement::Move}}, util::fen::{get_notation_for_piece, FenBoardState}};

use super::arguments::{ArgumentParser, Arguments};


fn build_argument_parser() -> ArgumentParser {
    let mut builder = ArgumentParser::builder();
    builder.add_subcommand("new").unwrap()
        .add_named_arg("from_fen", HashSet::from(["--from-fen"]), false, false).unwrap()
        .add_flag_arg("no_confirm", HashSet::from(["--no-confirm"])).unwrap();

    builder.add_subcommand("list").unwrap()
        .add_positional_arg("type", true, false).unwrap();

    builder.add_subcommand("suggest").unwrap()
        .add_positional_arg("count", false, false).unwrap();

    builder.add_subcommand("perft").unwrap()
        .add_named_arg("depth", HashSet::from(["--engine-depth"]), true, false).unwrap();

    builder.add_subcommand("move").unwrap();

    builder.add_subcommand("serialize").unwrap()
        .add_positional_arg("type", true, false).unwrap();

    builder.add_subcommand("board").unwrap()
        .add_flag_arg("as_fen", HashSet::from(["--as-fen"])).unwrap();

    builder.add_subcommand("search").unwrap()
        .add_named_arg("depth", HashSet::from(["--engine-depth"]), false, false).unwrap()
        .add_named_arg("threads", HashSet::from(["--threads"]), false, false).unwrap();

    builder.add_subcommand("exit").unwrap();

    return builder.build();
}


fn get_text_for_move(mov: &Move) -> String {
    return match mov {
        Move::NewGame(_) => {
            String::from("new game")
        },
        Move::Castle(c) => {
            format!("{} castles {}", c.color.value(), c.side.value())
        },
        _ => {
            let movement = mov.get_piece_movements()[0];
            let piece_text = format!("{} {} on {}", movement.color.value(), movement.piece_type.value(), get_notation_string_for_square(movement.start_square).unwrap());
            let movement_text = match mov.get_capture() {
                Some(c) => format!("captures {} {} on", c.color.value(), c.piece_type.value()),
                None => String::from("moves to"),
            };
            let result_text = match mov {
                Move::EnPassant(e) => format!("{} en passant, moving to {}", get_notation_string_for_square(e.capture_square).unwrap(), get_notation_string_for_square(movement.end_square).unwrap()),
                Move::Promotion(p) => format!("{} and promotes to a {}", get_notation_string_for_square(movement.end_square).unwrap(), p.promote_to.value()),
                _ => format!("{}", get_notation_string_for_square(movement.end_square).unwrap()),
            };
            format!("{} {} {}", piece_text, movement_text, result_text)
        }
    }
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
            game: Game::new(),
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
                        "new"           => self.do_new(*s.args),
                        "list"          => self.do_list(*s.args),
                        "move"          => self.do_move(*s.args),
                        "perft"         => self.do_perft(*s.args),
                        "search"        => self.do_search(*s.args),
                        "serialize"     => self.do_serialize(*s.args),
                        "board"         => self.do_board(*s.args),
                        "exit"          => break,
                        x => println!("Unknown subcommand {} encountered", x)
                    },
                    ParsedArgs::Arguments(a) => {
                        self.do_default(a)
                    }
                }
            };
            self.shell.empty_line();
        }
        self.shell.output("Exiting...");
        self.shell.empty_line();
    }

    fn do_default(&self, _args: Arguments) {

    }
    
    fn do_new(&mut self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'new' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                let mut confirmed = a.get_flag("no_confirm");
                if !confirmed {
                    let confirm = self.shell.input("Are you sure you want to start a new game? All progress on the current game will be lost. (y/N): ");
                    confirmed = self.confirmations.contains(&confirm.to_lowercase());
                }
                if confirmed {
                    match a.get_arg("from_fen") {
                        Some(fen) => self.game = Game::from_fen(&fen),
                        None => self.game = Game::new()
                    }
                    self.shell.output("New game started!");
                } else {
                    self.shell.output("OK, aborting...");
                }
            }
        }
    }
    
    fn do_list(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'list' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                match a.get_arg("type").unwrap().as_str() {
                    "moves" => {
                        for m in self.game.get_legal_moves() {
                            self.shell.output(&get_text_for_move(&m));
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
                        self.shell.output(&get_text_for_move(&m));
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
    
    fn do_perft(&mut self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'size' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                match a.get_arg("depth") {
                    Some(arg) => {
                        let depth: u8 = arg.parse().unwrap();
                        let result = self.game.do_perft(depth);
                        let table = Table::new(result.get_analysis()).with(Style::pseudo_clean());
                        self.shell.output(&table.to_string());
                        self.shell.output(&format!("Completed in {:?}", result.duration));
                        self.shell.empty_line();
                        self.shell.output(&format!("Starting Zobrist ID: {}", result.zobrist_start));
                        self.shell.output(&format!("Ending Zobrist ID:   {}", result.zobrist_end));

                    },
                    None => self.shell.output("Missing required field: 'depth' (use '--engine-depth')")
                }
            }
        }
    }

    fn do_board(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'board' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                let fen = self.game.serialize_board();
                match a.get_flag("as_fen") {
                    true => self.shell.output(&fen),
                    false => {
                        FenBoardState::from_fen(&fen).board.iter().for_each(|row| {
                            self.shell.output(&format!("{}", row.iter().map(|square| {
                                match square {
                                    Some((c, p)) => get_notation_for_piece(*c, *p).to_string(),
                                    None => String::from("-"),
                                }
                            }).collect::<Vec<String>>().join(" ")));
                        })
                    }
                }
            }
        }
    }

    fn do_search(&mut self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'search' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                let depth: u8 = match a.get_arg("depth") {
                    Some(d) => d.parse().unwrap(),
                    None => self.shell.input("What depth should the engine search to? ").parse().unwrap()
                };
                self.shell.output("Searching...");
                let result = match a.get_arg("threads") {
                    Some(t) => self.game.threaded_search(depth, t.parse().unwrap()),
                    None => self.game.search(depth),
                };
                self.shell.output("Search complete!");
                self.shell.empty_line();
                self.shell.output(&format!("Evaluated {} nodes with {} cache hits in {:?}", result.calculated_nodes, result.cache_hits, result.search_time));
                self.shell.output(&format!("Score: {}", result.get_score()));
                self.shell.output(&format!("Best move: {}", &get_text_for_move(result.get_move())));
            }
        }
    }

    fn do_serialize(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'serialize' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                match a.get_arg("type") {
                    Some(arg) => match arg.as_str() {
                        "board" => self.shell.output(&self.game.serialize_board()),
                        _ => ()
                    },
                    None => ()
                }
            }
        }
    }

    fn get_promotion_choice(&self) -> PieceType {
        self.shell.output("What should it promote to?");
        self.shell.output("    1. Queen");
        self.shell.output("    2. Rook");
        self.shell.output("    3. Bishop");
        self.shell.output("    4. Knight");
        let choice = self.shell.input("? ").clone();
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
}
