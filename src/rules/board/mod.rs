pub mod bitboards;
pub mod positions;
pub mod squares;
pub mod state;

use fxhash::FxHashMap;

use crate::rules::board::positions::CastlingSquares;
use crate::util::fen::{FenBoardState, Castling, STARTING_POSITION};
use crate::util::zobrist::ZobristId;

use self::bitboards::{BitboardSquares, get_bit_for_square, get_moves_for_piece};
use self::positions::{BoardPosition, Pin, AttacksAndPins, Attack, PieceLocation};
use self::squares::{BoardSquare, get_col_and_row_from_square, get_square_from_col_and_row, is_fourth_rank, is_eighth_rank, is_second_rank};
use self::state::{CastleRight, BoardState, BoardCastles, ReversibleBoardChange, ApplyableBoardChange};

use super::Color;
use super::pieces::{Piece, PieceType};
use super::pieces::movement::{BasicMove, Castle, CastleType, EnPassant, Move, Promotion, TwoSquarePawnMove};


lazy_static! {
    static ref CASTLING_MOVES: FxHashMap<Color, FxHashMap<CastleType, CastlingSquares>> = FxHashMap::from_iter([
        (Color::White, FxHashMap::from_iter([
            (CastleType::Kingside, CastlingSquares::from_color_and_type(Color::White, CastleType::Kingside)),
            (CastleType::Queenside, CastlingSquares::from_color_and_type(Color::White, CastleType::Queenside))
        ].into_iter())),
        (Color::Black, FxHashMap::from_iter([
            (CastleType::Kingside, CastlingSquares::from_color_and_type(Color::Black, CastleType::Kingside)),
            (CastleType::Queenside, CastlingSquares::from_color_and_type(Color::Black, CastleType::Queenside))
        ].into_iter())),
    ].into_iter());
}


impl CastleRight {
    fn associated_rights_by_square(square: u8) -> Vec<Self> {
        let mut rights = Vec::new();
        if square == BoardSquare::A1.value() || square == BoardSquare::E1.value() {
            rights.push(Self { color: Color::White, side: CastleType::Queenside });
        }
        if square == BoardSquare::H1.value() || square == BoardSquare::E1.value() {
            rights.push(Self { color: Color::White, side: CastleType::Kingside });
        }
        if square == BoardSquare::A8.value() || square == BoardSquare::E8.value() {
            rights.push(Self { color: Color::Black, side: CastleType::Queenside });
        }
        if square == BoardSquare::H8.value() || square == BoardSquare::E8.value() {
            rights.push(Self { color: Color::Black, side: CastleType::Kingside });
        }
        return rights;
    }
}


fn get_capture_square_for_ep_target(ep_target: u8) -> u8 {
    return if ep_target > 31 { ep_target - 8 } else { ep_target + 8 };
}

fn get_en_passant_target_for_two_square_first_move(color: Color, square: u8) -> u8 {
    let [col, row] = get_col_and_row_from_square(square);
    let direction: i8 = match color { Color::White => -1, Color::Black => 1 };
    return get_square_from_col_and_row(col, (row as i8 + direction) as u8)
}

fn piece_map_from_fen_board(board: [[Option<Piece>; 8]; 8]) -> FxHashMap<u8, Piece> {
    let mut piece_map: FxHashMap<u8, Piece> = Default::default();
    board.into_iter().rev().enumerate().for_each(|(row_index, row)| {
        row.into_iter().enumerate().for_each(|(col_index, option)| {
            match option {
                None => (),
                Some(piece) => {
                    piece_map.insert((col_index + (row_index * 8)) as u8, Piece { color: piece.color, piece_type: piece.piece_type });
                    ();
                }
            }
        })
    });
    return piece_map;
}

pub fn fen_board_from_position(position: &BoardPosition) -> [[Option<Piece>; 8]; 8] {
    let mut board: [[Option<Piece>; 8]; 8] = Default::default();
    for color in Color::iter() {
        for ps in position.get_all_masked_piece_squares_for_color(color, u64::MAX) {
            let [col, row] = get_col_and_row_from_square(ps.square);
            let fen_row = 7 - row;
            board[fen_row as usize][col as usize] = Some(ps.piece);
        }
    }
    return board;
}


fn board_from_fen_state(state: FenBoardState) -> Board {
    let piece_map = piece_map_from_fen_board(state.board);
    return Board {
        position: BoardPosition::from_piece_map(piece_map.clone()),
        state: BoardState {
            to_move: state.to_move,
            en_passant_target: match state.en_passant {
                Some(s) => get_bit_for_square(s.value()),
                None => 0u64
            },
            halfmove_clock: state.halfmove_timer,
            move_number: state.move_number,
            castle_rights: BoardCastles {
                white_kingside: state.castling.white_kingside,
                white_queenside: state.castling.white_queenside,
                black_kingside: state.castling.black_kingside,
                black_queenside: state.castling.black_queenside,
            }
        },
        zobrist: ZobristId::from_fen(&state),
    }
}

fn fen_state_from_board(board: &Board) -> FenBoardState {
    return FenBoardState {
        board: fen_board_from_position(&board.position),
        to_move: board.state.to_move,
        castling: Castling {
            white_kingside: board.state.castle_rights.white_kingside,
            white_queenside: board.state.castle_rights.white_queenside,
            black_kingside: board.state.castle_rights.black_kingside,
            black_queenside: board.state.castle_rights.black_queenside,
        },
        en_passant: match board.state.en_passant_target {
            0 => None,
            x => Some(BoardSquare::from_value(x.trailing_zeros() as u8))
        },
        halfmove_timer: board.state.halfmove_clock,
        move_number: board.state.move_number,
    }
}


fn get_castle_details(color: Color, castle_type: CastleType) -> &'static CastlingSquares {
    return CASTLING_MOVES.get(&color).unwrap().get(&castle_type).unwrap()
}


fn get_legal_king_moves(position: &BoardPosition, color: Color) -> Vec<Move> {
    let king_square = position.find_king(color);
    return get_moves_for_piece_location(position, &PieceLocation {square: king_square, piece: Piece { color: color, piece_type: PieceType::King }}, 0u64);
}

fn get_legal_moves_from_check(position: &BoardPosition, color: Color, check: &Attack, pinned: u64, ep_target: u64) -> Vec<Move> {
    let mut moves = get_legal_king_moves(position, color);
    for loc in position.get_all_masked_piece_squares_for_color(color, !pinned) {
        if loc.piece.piece_type == PieceType::King { continue };
        let move_board = get_moves_for_piece(
            loc.square,
            loc.piece,
            position.get_all_piece_locations(color),
            position.get_all_piece_locations(color.swap()),
            ep_target);
        let legal_moves = move_board & (check.attack_path | get_bit_for_square(check.attacking_square));
        for end_square in BitboardSquares::from_board(legal_moves) {
            moves.extend(build_move(position, loc.square, end_square, &loc.piece, ep_target));
        }
        if loc.piece.piece_type == PieceType::Pawn && move_board & ep_target != 0 {
            let end = ep_target.trailing_zeros() as u8;
            let capture_square = get_capture_square_for_ep_target(end);
            if capture_square == check.attacking_square && move_board & get_bit_for_square(end) != 0 {
                moves.extend(build_move(position, loc.square, end, &loc.piece, ep_target))
            }
        }
    }
    return moves;
}

fn get_legal_moves_for_pinned_piece(position: &BoardPosition, pin: &Pin, ep_target: u64) -> Vec<Move> {
    let pinned_piece = position.piece_at(&pin.pinned_square).unwrap();
    let move_board = get_moves_for_piece(
        pin.pinned_square,
        pinned_piece,
        position.get_all_piece_locations(pinned_piece.color),
        position.get_all_piece_locations(pinned_piece.color.swap()),
        ep_target);
    let legal_moves = move_board & (pin.pin_path | get_bit_for_square(pin.pinning_square));
    BitboardSquares::from_board(legal_moves).fold(Vec::new(), |mut moves, s| {
        moves.extend(build_move(position, pin.pinned_square, s, &pinned_piece, ep_target));
        moves
    })
}

fn get_moves_for_piece_location(position: &BoardPosition, loc: &PieceLocation, ep_target: u64) -> Vec<Move> {
    let move_board = get_moves_for_piece(
        loc.square,
        loc.piece,
        position.get_all_piece_locations(loc.piece.color),
        position.get_all_piece_locations(loc.piece.color.swap()),
        ep_target);
    BitboardSquares::from_board(move_board).fold(Vec::new(), |mut moves, s| {
        moves.extend(build_move(position, loc.square, s, &loc.piece, ep_target));
        moves
    })
}

fn build_move(position: &BoardPosition, start: u8, end: u8, piece: &Piece, ep_target: u64) -> Vec<Move> {
    let capture = position.piece_at(&end).map(|p| p);
    let basic_move = BasicMove { piece: *piece, start: start, end: end, capture: capture };
    if piece.piece_type == PieceType::King && position.is_check(end, piece.color) {
        return Vec::new();
    }
    if piece.piece_type == PieceType::Pawn && end == ep_target.trailing_zeros() as u8 {
        let capture_square = get_capture_square_for_ep_target(end);
        if position.en_passant_is_illegal(piece.color, start, end, capture_square) {
            return Vec::new();
        } else {
            let ep_capture = position.piece_at(&capture_square).map(|p| p);
            let mut ep_basic = basic_move.clone();
            ep_basic.capture = ep_capture;
            match ep_capture {
                Some(_) => return Vec::from([ Move::EnPassant(EnPassant::from_basic_move(&ep_basic, capture_square)) ]),
                None => panic!("Invalid Move: Cannot create an en passant move without a capture!"),
            }
        }
    } else if piece.piece_type == PieceType::Pawn && is_eighth_rank(end, piece.color) {
        return Promotion::get_all_from_basic_move(&basic_move).into_iter().map(|p| { Move::Promotion(p) }).collect()
    } else if piece.piece_type == PieceType::Pawn && is_second_rank(start, piece.color) && is_fourth_rank(end, piece.color) {
        if capture.is_some() { panic!("Invalid Move: Cannot create a two square pawn move with a captured piece!") }
        let en_passant_target = get_en_passant_target_for_two_square_first_move(piece.color, end);
        return Vec::from([ Move::TwoSquarePawnMove(TwoSquarePawnMove::from_basic_move(&basic_move, en_passant_target)) ]);
    } else {
        return Vec::from([ Move::BasicMove(basic_move) ]);
    }
}


fn predict_lost_castle_rights(mov: &Move, state: &BoardState) -> Vec<CastleRight> {
    let mut rights = Vec::new();
    for movement in mov.get_piece_movements() {
        rights.extend(CastleRight::associated_rights_by_square(movement.start_square));
        rights.extend(CastleRight::associated_rights_by_square(movement.end_square));
    }
    return rights.into_iter().filter(|r| state.can_castle(r)).collect();
}


fn predict_zobrist_update(old_id: &ZobristId, mov: &Move, revoked_castle_rights: &Vec<CastleRight>, state: &BoardState) -> ZobristId {
    let mut new_id = *old_id;
    new_id.update_move(&mov);
    for right in revoked_castle_rights {
        new_id.update_castle_right(*right);
    }
    if let Some(ep) = state.get_en_passant_target() {
        new_id.update_en_passant(ep);
    }
    if let Move::TwoSquarePawnMove(m) = mov {
        new_id.update_en_passant(m.en_passant_target);
    }
    new_id.update_turn();
    return new_id;
}


fn prepare_change(mov: Move, position: &BoardPosition, state: &BoardState, board_id: ZobristId) -> ApplyableBoardChange {
    let mut updated_position = *position;
    let mut updated_state = *state;
    updated_position.apply_move(&mov);
    let checks_and_pins = updated_position.get_attacks_and_pins(
        updated_position.find_king(updated_state.get_move_color().swap()),
        updated_state.get_move_color().swap());
    let revoked_castle_rights = predict_lost_castle_rights(&mov, &updated_state);
    for right in &revoked_castle_rights {
        updated_state.revoke_castle_right(right);
    }
    if let Move::TwoSquarePawnMove(m) = mov {
        updated_state.set_en_passant_target(m.en_passant_target);
    } else {
        updated_state.clear_en_passant_target();
    }
    updated_state.increment_halfmove_clock();
    if let Some(_capture) = mov.get_capture() {
        updated_state.reset_halfmove_clock();
    } else if mov.get_piece_movements()[0].get_piece().piece_type == PieceType::Pawn {
        updated_state.reset_halfmove_clock();
    }
    if updated_state.get_move_color() == Color::Black { updated_state.increment_move_number(); }
    updated_state.change_move_color();

    let updated_zobrist_id = predict_zobrist_update(&board_id, &mov, &revoked_castle_rights, &updated_state);

    let responses: Vec<ApplyableBoardChange> = (if checks_and_pins.attacks.len() > 1 {
        get_legal_king_moves(&updated_position, updated_state.get_move_color())
    } else if !checks_and_pins.attacks.is_empty() {
        get_legal_moves_from_check(
            &updated_position,
            updated_state.get_move_color(),
            &checks_and_pins.attacks[0],
            checks_and_pins.pinned,
            updated_state.en_passant_target)
    } else {
        Vec::new()
    }).into_iter().map(|m| { prepare_change(m, &updated_position, &updated_state, updated_zobrist_id) }).collect();

    return ApplyableBoardChange {
        new_move: mov,
        checks: checks_and_pins.attacks,
        absolute_pins: checks_and_pins.pins,
        pinned_pieces: checks_and_pins.pinned,
        responses: responses,
        new_zobrist_id: updated_zobrist_id,
        new_position: updated_position,
        new_state: updated_state,
    }
}


fn get_pinned_moves_closure(pin: &Pin, position: &BoardPosition, state: &BoardState, id: ZobristId) -> Box<dyn FnOnce() -> Vec<ApplyableBoardChange> + Send> {
    let owned_pin = *pin;
    let owned_position = *position;
    let owned_state = *state;
    let closure = move || {
        get_legal_moves_for_pinned_piece(&owned_position, &owned_pin, owned_state.en_passant_target).into_iter().map(|m| {
            prepare_change(m, &owned_position, &owned_state, id)
        }).collect()
    };
    return Box::new(closure);
}


fn get_piece_moves_closure(piece_square: &PieceLocation, position: &BoardPosition, state: &BoardState, id: ZobristId) -> Box<dyn FnOnce() -> Vec<ApplyableBoardChange> + Send> {
    let owned_piece_square = *piece_square;
    let owned_position = *position;
    let owned_state = *state;
    let closure = move || {
        get_moves_for_piece_location(&owned_position, &owned_piece_square, owned_state.en_passant_target).into_iter().map(|m| {
            prepare_change(m, &owned_position, &owned_state, id)
        }).collect()
    };
    return Box::new(closure);
}


#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Board {
    pub position: BoardPosition,
    pub state: BoardState,
    pub zobrist: ZobristId,
}

impl Board {
    pub fn from_starting_position() -> Board {
        return Self::from_fen(STARTING_POSITION);
    }

    pub fn from_fen(fen: &str) -> Board {
        let parsed_fen = FenBoardState::from_fen(fen);
        return board_from_fen_state(parsed_fen);
    }

    pub fn to_fen(&self) -> String {
        return fen_state_from_board(self).to_fen();
    }

    pub fn repeats(&self, other: &Self) -> bool {
        return self.position == other.position &&
            self.state.to_move == other.state.to_move &&
            self.state.en_passant_target == other.state.en_passant_target &&
            self.state.castle_rights == other.state.castle_rights;
    }

    pub fn get_legal_moves(&self) -> Vec<Move> {
        let king_square = self.find_king(self.state.to_move);
        let mut moves: Vec<Move> = Vec::new();
        let checks_and_pins = self.get_checks_and_pins(&king_square, self.state.to_move);
        let checks = checks_and_pins.attacks;
        let pins = checks_and_pins.pins;
        let pinned_squares = checks_and_pins.pinned;
        if checks.len() > 1 { return self.get_legal_king_moves() }
        if !checks.is_empty() { return self.get_legal_moves_from_check(checks.first().unwrap(), pinned_squares) }
        for pin in pins {
            moves.extend(self.get_legal_moves_for_pinned_piece(&pin))
        }
        self.position.get_all_masked_piece_squares_for_color(self.state.to_move, !pinned_squares).for_each(|loc| {
            moves.extend(self.get_moves_for_piece_location(loc))
        });
        moves.extend(self.get_castle_moves(self.state.to_move));
        return moves;
    }

    pub fn make_move(&mut self, new_move: &Move) -> ReversibleBoardChange {
        let result = ReversibleBoardChange {
            prior_zobrist_id: self.zobrist,
            prior_position: self.position,
            prior_state: self.state,
        };
        self.zobrist.update_move(new_move);
        for castle in self.revoke_castle_rights(new_move) {
            self.zobrist.update_castle_right(castle);
        }
        match self.state.clear_en_passant_target() {
            Some(square) => self.zobrist.update_en_passant(square),
            None => (),
        }
        self.state.increment_halfmove_clock();
        if let Some(_capture) = new_move.get_capture() {
            self.state.reset_halfmove_clock();
        } else if new_move.get_piece_movements()[0].get_piece().piece_type == PieceType::Pawn {
            self.state.reset_halfmove_clock();
        }
        self.position.apply_move(new_move);
        
        match new_move {
            Move::TwoSquarePawnMove(m) => {
                self.state.set_en_passant_target(m.en_passant_target);
                self.zobrist.update_en_passant(m.en_passant_target);
            },
            _ => ()
        }
        if self.state.get_move_color() == Color::Black { self.state.increment_move_number(); }
        self.state.change_move_color();
        self.zobrist.update_turn();

        return result;
    }

    pub fn unmake_move(&mut self, change: ReversibleBoardChange) {
        self.zobrist = change.prior_zobrist_id;
        self.position = change.prior_position;
        self.state = change.prior_state;
    }

    fn revoke_castle_rights(&mut self, new_move: &Move) -> Vec<CastleRight> {
        let mut rights = Vec::new();
        for movement in new_move.get_piece_movements() {
            rights.extend(CastleRight::associated_rights_by_square(movement.start_square));
            rights.extend(CastleRight::associated_rights_by_square(movement.end_square));
        }
        return rights.into_iter().filter_map(|r| {
            match self.state.can_castle(&r) {
                true => {
                    self.state.revoke_castle_right(&r);
                    Some(r)
                },
                false => None,
            }
        }).collect();
    }

    fn get_moves_for_piece(&self, square: u8) -> Vec<Move> {
        let piece = self.position.piece_at(&square).unwrap();
        return self.get_moves_for_piece_location( PieceLocation { square: square, piece: piece } )
    }

    fn get_moves_for_piece_location(&self, loc: PieceLocation) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();
        let move_board = get_moves_for_piece(
            loc.square,
            loc.piece,
            self.position.get_all_piece_locations(loc.piece.color),
            self.position.get_all_piece_locations(loc.piece.color.swap()),
            self.state.en_passant_target);
        BitboardSquares::from_board(move_board).for_each(|end_square| {
            moves.extend(self.build_move(loc.square, end_square, loc.piece))
        });
        return moves;
    }

    fn get_legal_moves_for_pinned_piece(&self, pin: &Pin) -> Vec<Move> {
        let pinned_piece = self.position.piece_at(&pin.pinned_square).unwrap();
        let move_board = get_moves_for_piece(
            pin.pinned_square,
            pinned_piece,
            self.position.get_all_piece_locations(pinned_piece.color),
            self.position.get_all_piece_locations(pinned_piece.color.swap()),
            self.state.en_passant_target);
        let legal_moves = move_board & (pin.pin_path | get_bit_for_square(pin.pinning_square));
        BitboardSquares::from_board(legal_moves).fold(Vec::new(), |mut moves, s| {
            moves.extend(self.build_move(pin.pinned_square, s, pinned_piece));
            moves
        })
    }

    fn get_legal_moves_from_check(&self, check: &Attack, pinned_squares: u64) -> Vec<Move> {
        let mut moves = self.get_legal_king_moves();
        let pieces = self.get_pieces_to_move() ^ pinned_squares;
        for start_square in BitboardSquares::from_board(pieces) {
            let piece = self.position.piece_at(&start_square).unwrap();
            if piece.piece_type == PieceType::King { continue };
            let move_board = get_moves_for_piece(
                start_square,
                piece,
                self.position.get_all_piece_locations(piece.color),
                self.position.get_all_piece_locations(piece.color.swap()),
                self.state.en_passant_target);
            let legal_moves = move_board & (check.attack_path | get_bit_for_square(check.attacking_square));
            for end_square in BitboardSquares::from_board(legal_moves) {
                moves.extend(self.build_move(start_square, end_square, piece));
            }
            if piece.piece_type == PieceType::Pawn && move_board & self.state.en_passant_target != 0 {
                let end = self.state.get_en_passant_target().unwrap();
                let capture_square = get_capture_square_for_ep_target(end);
                if capture_square == check.attacking_square && move_board & get_bit_for_square(end) != 0 {
                    moves.extend(self.build_move(start_square, end, piece))
                }
            }
        }
        return moves;
    }

    fn get_legal_king_moves(&self) -> Vec<Move> {
        return self.get_moves_for_piece(self.position.find_king(self.state.get_move_color()));
    }

    fn get_castle_moves(&self, color: Color) -> Vec<Move> {
        [self.get_castle(color, CastleType::Kingside), self.get_castle(color, CastleType::Queenside)].into_iter().filter_map(|opt| {
            match opt { Some(m) => Some(m), None => None }
        }).collect()
    }

    fn get_castle(&self, color: Color, side: CastleType) -> Option<Move> {
        if !self.state.can_castle(&CastleRight{ color: color, side: side }) { return None };
        let detail = get_castle_details(color, side);
        let all_pieces = self.position.get_all_piece_locations(Color::White) | self.position.get_all_piece_locations(Color::Black);
        if detail.transit_squares & all_pieces != 0 { return None };
        for square in BitboardSquares::from_board(detail.king_transit_squares) {
            if self.position.is_check(square, color) { return None }
        }
        if self.position.is_check(detail.king_end, color) { return None };
        return Some(Move::Castle(Castle {
            color: color,
            side: side,
            king_start: detail.king_start,
            king_end: detail.king_end,
            rook_start: detail.rook_start,
            rook_end: detail.rook_end,
        }))
    }

    fn find_king(&self, color: Color) -> u8 {
        return match color {
            Color::White => self.position.find_king(Color::White),
            Color::Black => self.position.find_king(Color::Black),
        }
    }

    fn get_pieces_to_move(&self) -> u64 {
        return match self.state.get_move_color() {
            Color::White => self.position.get_all_piece_locations(Color::White),
            Color::Black => self.position.get_all_piece_locations(Color::Black),
        }
    }

    pub fn in_check(&self) -> bool {
        return self.position.is_check(self.position.find_king(self.state.get_move_color()), self.state.get_move_color())
    }

    fn get_checks_and_pins(&self, king_square: &u8, king_color: Color) -> AttacksAndPins {
        return self.position.get_attacks_and_pins(*king_square, king_color);
    }

    fn build_move(&self, start: u8, end: u8, piece: Piece) -> Vec<Move> {
        let capture = self.position.piece_at(&end).map(|p| p);
        let basic_move = BasicMove { piece: piece, start: start, end: end, capture: capture };
        if piece.piece_type == PieceType::King && self.position.is_check(end, piece.color) {
            return Vec::new();
        }
        if piece.piece_type == PieceType::Pawn && end == self.state.get_en_passant_target().unwrap_or(255) {
            let capture_square = get_capture_square_for_ep_target(end);
            if self.position.en_passant_is_illegal(piece.color, start, end, capture_square) {
                return Vec::new();
            } else {
                let ep_capture = self.position.piece_at(&capture_square).map(|p| p);
                let mut ep_basic = basic_move.clone();
                ep_basic.capture = ep_capture;
                match ep_capture {
                    Some(_) => return Vec::from([ Move::EnPassant(EnPassant::from_basic_move(&ep_basic, capture_square)) ]),
                    None => panic!("Invalid Move: Cannot create an en passant move without a capture!"),
                }
            }
        } else if piece.piece_type == PieceType::Pawn && is_eighth_rank(end, piece.color) {
            return Promotion::get_all_from_basic_move(&basic_move).into_iter().map(|p| { Move::Promotion(p) }).collect()
        } else if piece.piece_type == PieceType::Pawn && is_second_rank(start, piece.color) && is_fourth_rank(end, piece.color) {
            if capture.is_some() { panic!("Invalid Move: Cannot create a two square pawn move with a captured piece!") }
            let en_passant_target = get_en_passant_target_for_two_square_first_move(piece.color, end);
            return Vec::from([ Move::TwoSquarePawnMove(TwoSquarePawnMove::from_basic_move(&basic_move, en_passant_target)) ]);
        } else {
            return Vec::from([ Move::BasicMove(basic_move) ]);
        }
    }
}