use std::{collections::HashSet, time::Instant};

use num_format::{ToFormattedString, Locale};
use tabled::{Table, Style, Alignment, Modify, Full};

use crate::{game::Game, interface::{arguments::ParsedArgs, shell::InteractiveShell}, rules::{board::{squares::{BoardSquare, get_notation_string_for_square}, fen_board_from_position, Board}, pieces::{PieceType, movement::Move, Piece}, Color}, util::{fen::{FenBoardState, get_notation_for_piece}, zobrist::ZobristId}, testing::{perft::PerftRunner, zobrist::ZobristCollisionTester}, engine::search::alpha_beta::AlphaBetaSearch};

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
        .add_named_arg("depth", HashSet::from(["--depth"]), true, false).unwrap()
        .add_named_arg("threads", HashSet::from(["--threads"]), false, false).unwrap();

    builder.add_subcommand("zobrist_test").unwrap()
        .add_named_arg("depth", HashSet::from(["--depth"]), true, false).unwrap();

    builder.add_subcommand("move").unwrap();

    builder.add_subcommand("serialize").unwrap()
        .add_positional_arg("type", true, false).unwrap();

    builder.add_subcommand("board").unwrap()
        .add_flag_arg("hide_board", HashSet::from(["--hide-board"])).unwrap()
        .add_flag_arg("as_fen", HashSet::from(["--as-fen"])).unwrap()
        .add_flag_arg("as_zobrist", HashSet::from(["--as-zobrist"])).unwrap();

    builder.add_subcommand("search").unwrap()
        .add_named_arg("depth", HashSet::from(["--depth"]), false, false).unwrap()
        .add_named_arg("threads", HashSet::from(["--threads"]), false, false).unwrap()
        .add_named_arg("sleep", HashSet::from(["--sleep"]), false, false).unwrap();

    builder.add_subcommand("exit").unwrap();

    return builder.build();
}


fn get_text_for_move(mov: &Move) -> String {
    return match mov {
        Move::NullMove(_) => {
            String::from("null move")
        },
        Move::Castle(c) => {
            format!("{} castles {}", c.color.value(), c.side.value())
        },
        _ => {
            let movement = mov.get_piece_movements()[0];
            let piece_text = format!("{} {} on {}", movement.color.value(), movement.piece_type.name(), get_notation_string_for_square(movement.start_square).unwrap());
            let movement_text = match mov.get_capture() {
                Some(c) => format!("captures {} {} on", c.color.value(), c.piece_type.name()),
                None => String::from("moves to"),
            };
            let result_text = match mov {
                Move::EnPassant(e) => format!("{} en passant, moving to {}", get_notation_string_for_square(e.capture_square).unwrap(), get_notation_string_for_square(movement.end_square).unwrap()),
                Move::Promotion(p) => format!("{} and promotes to a {}", get_notation_string_for_square(movement.end_square).unwrap(), p.promote_to.name()),
                _ => format!("{}", get_notation_string_for_square(movement.end_square).unwrap()),
            };
            format!("{} {} {}", piece_text, movement_text, result_text)
        }
    }
}


fn get_unicode_piece_symbol(piece: &Piece) -> String {
    let char_type = std::str::from_utf8(&[0b11101111 as u8, 0b10111000 as u8, 0b10001110 as u8]).unwrap();
    match (piece.color, piece.piece_type) {
        (Color::White, PieceType::Pawn)   => format!("{}{}", "???", char_type), (Color::Black, PieceType::Pawn)   => format!("{}{}", "???", char_type),
        (Color::White, PieceType::Knight) => format!("{}{}", "???", char_type), (Color::Black, PieceType::Knight) => format!("{}{}", "???", char_type),
        (Color::White, PieceType::Bishop) => format!("{}{}", "???", char_type), (Color::Black, PieceType::Bishop) => format!("{}{}", "???", char_type),
        (Color::White, PieceType::Rook)   => format!("{}{}", "???", char_type), (Color::Black, PieceType::Rook)   => format!("{}{}", "???", char_type),
        (Color::White, PieceType::Queen)  => format!("{}{}", "???", char_type), (Color::Black, PieceType::Queen)  => format!("{}{}", "???", char_type),
        (Color::White, PieceType::King)   => format!("{}{}", "???", char_type), (Color::Black, PieceType::King)   => format!("{}{}", "???", char_type),
    }
}


fn format_board_for_display(board: &Board) -> Vec<String> {
    let mut result = Vec::new();
    result.push(String::from("  ???????????????????????????????????????????????????????????????????????????????????????????????????"));
    fen_board_from_position(&board.position).iter().enumerate().for_each(|(index, row)| {
        result.push(row.iter().fold(format!("{}{}", (7 - index) + 1, " ???"), |row_string, square| {
            format!("{} {} ???", row_string, match square {
                Some(piece) => get_notation_for_piece(*piece).to_string(),
                None => String::from(" "),
            })
        }));
        result.push(String::from("  ???????????????????????????????????????????????????????????????????????????????????????????????????"));
    });
    result.pop();
    result.push(String::from("  ???????????????????????????????????????????????????????????????????????????????????????????????????"));
    result.push(String::from("    a   b   c   d   e   f   g   h  "));
    return result;
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
                        "zobrist_test"  => self.do_zobrist_test(*s.args),
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
                Move::NullMove(_) => ()
            }
        }
        return chosen_move;
    }
    
    fn do_perft(&mut self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'perft' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                let depth: u8 = match a.get_arg("depth") {
                    Some(d) => d.parse().unwrap(),
                    None => self.shell.input("What depth should the engine search to? ").parse().unwrap()
                };
                let start = Instant::now();
                let result = match a.get_arg("threads") {
                    Some(t) => PerftRunner::do_threaded_perft(*self.game.get_board(), depth, t.parse().unwrap()),
                    None => PerftRunner::do_perft(*self.game.get_board(), depth),
                };
                let duration = start.elapsed();
                let table = Table::new(result.get_analysis()).with(Style::pseudo_clean()).with(Modify::new(Full).with(Alignment::right()));
                self.shell.output(&table.to_string());
                self.shell.output(&format!("Completed in {:?}", duration));
            }
        }
    }

    fn do_zobrist_test(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'zobrist_test' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                let depth: u8 = match a.get_arg("depth") {
                    Some(d) => d.parse().unwrap(),
                    None => self.shell.input("What depth should we test to? ").parse().unwrap()
                };
                let results = ZobristCollisionTester::do_test(*self.game.get_board(), depth);
                self.shell.empty_line();
                self.shell.output(&format!("Total Positions Evaluated: {}", results.positions_checked.to_formatted_string(&Locale::en)));
                self.shell.output(&format!("          Evaluation Time: {:?}", results.duration));
                self.shell.output(&format!("        Memory Space Used: {}", bytefmt::format(results.memory_size as u64)));
                self.shell.empty_line();
                self.shell.output(&format!("    Zobrist Matches Found: {}", results.hash_matches.to_formatted_string(&Locale::en)));
                self.shell.output(&format!("True Transpositions Found: {}", results.transpositions.to_formatted_string(&Locale::en)));
                self.shell.output(&format!("         Collisions Found: {}", results.collisions.to_formatted_string(&Locale::en)));
                self.shell.output(&format!("      Collided Hash Count: {}", results.collided_hash_count.to_formatted_string(&Locale::en)));
                self.shell.empty_line();

                for collision in results.collision_pairs {
                    let mut old = collision.fen_2;
                    let mut new = collision.fen_1;
                    self.shell.output("Collision:");
                    old.push_str(" 0 0");
                    new.push_str(" 0 0");
                    self.shell.output(&format!("             Move: {}", get_text_for_move(&collision.cause)));
                    self.shell.output(&format!("            Fen 1: {}", old));
                    self.shell.output(&format!("            Fen 2: {}", new));
                    self.shell.output(&format!("           Hash 1: {:064b}", ZobristId::from_fen(&FenBoardState::from_fen(&old)).get_id()));
                    self.shell.output(&format!("    Collided Hash: {:064b}", collision.hash));
                    self.shell.output(&format!("           Hash 2: {:064b}", ZobristId::from_fen(&FenBoardState::from_fen(&new)).get_id()));
                    let old_board = format_board_for_display(&Board::from_fen(&old));
                    let new_board = format_board_for_display(&Board::from_fen(&new));
                    self.shell.output("        Board 1:               Board 2:");
                    old_board.into_iter().zip(new_board.into_iter()).for_each(|(old_row, new_row)| {
                        self.shell.output(&format!("    {}        {}", old_row, new_row));
                    });
                    self.shell.empty_line();
                }
            }
        }
    }

    fn do_board(&self, args: ParsedArgs) {
        match args {
            ParsedArgs::SubCommand(_s) => panic!("Subcommand 'board' should not have its own subcommands"),
            ParsedArgs::Arguments(a) => {
                if a.get_flag("as_fen") {
                    self.shell.output(&format!("Fen String:   {}", self.game.serialize_board()));
                    self.shell.empty_line();
                }
                if a.get_flag("as_zobrist") {
                    self.shell.output(&format!("Zobrist Hash: {:064b}", self.game.get_board().zobrist.get_id()));
                    self.shell.empty_line();
                }
                if !a.get_flag("hide_board") {
                    self.shell.output("    Board:");
                    format_board_for_display(self.game.get_board()).iter().for_each(|row| {
                        self.shell.output(row);
                    });
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
                let start = Instant::now();
                let result = match a.get_arg("threads") {
                    Some(t) => {
                        let threads: u8 = t.parse().unwrap_or(1);
                        let sleep = a.get_arg("sleep").unwrap_or("0".to_string()).parse().unwrap_or(0u64);
                        AlphaBetaSearch::do_threaded_search(*self.game.get_board(), depth, threads, sleep)
                    },
                    None => {
                        AlphaBetaSearch::do_search(*self.game.get_board(), depth)
                    }
                };
                let duration = start.elapsed();
                self.shell.empty_line();
                if let Some(mov) = result.mov {
                    self.shell.output(&format!("Best move: {}", get_text_for_move(&mov)));
                }
                self.shell.empty_line();
                self.shell.output(&format!("Position score:             {}", result.score));
                self.shell.output(&format!("Evaluated positions:        {}", result.evaluated_nodes.to_formatted_string(&Locale::en)));
                self.shell.output(&format!("Cached transpositions used: {}", result.cache_hits.to_formatted_string(&Locale::en)));
                self.shell.output(&format!("Beta cutoffs applied:       {}", result.beta_cutoffs.to_formatted_string(&Locale::en)));
                self.shell.output(&format!("Completed in:               {:?}", duration));
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
