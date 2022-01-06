use fxhash::FxHashMap;

use crate::rules::{pieces::{movement::{SlideDirection, PawnMovement}, Piece, PieceType}, Color};

use super::squares::BoardSquare;


lazy_static! {
    static ref RAY_BITBOARDS: FxHashMap<u16, u64> = prepare_ray_bitboards();
    static ref KNIGHT_BITBOARDS: FxHashMap<u8, u64> = prepare_knight_bitboards();
    static ref PAWN_BITBOARDS: FxHashMap<u8, u64> = prepare_pawn_bitboards();
    static ref KING_BITBOARDS: FxHashMap<u8, u64> = prepare_king_bitboards();

    static ref BISHOP_DIRS: Vec<SlideDirection> = Vec::from([
        SlideDirection::NorthEast, SlideDirection::SouthEast, SlideDirection::SouthWest, SlideDirection::NorthWest
    ]);
    static ref ROOK_DIRS: Vec<SlideDirection> = Vec::from([
        SlideDirection::North, SlideDirection::East, SlideDirection::South, SlideDirection::West
    ]);
    static ref QUEEN_DIRS: Vec<SlideDirection> = Vec::from([
        SlideDirection::NorthEast, SlideDirection::SouthEast, SlideDirection::SouthWest, SlideDirection::NorthWest,
        SlideDirection::North, SlideDirection::East, SlideDirection::South, SlideDirection::West
    ]);
}


pub fn get_bit_for_square(square: u8) -> u64 {
    return 2u64.pow(square as u32)
}


fn generate_sliding_bitboard(square: u8, direction: SlideDirection) -> u64 {
    let mut board = 0u64;
    let (col_shift, row_shift) = direction.get_direction();
    let mut current_square = square;
    loop {
        match BoardSquare::from_value(current_square).apply_movement(col_shift, row_shift) {
            Err(_) => break,
            Ok(new_square) => {
                board |= get_bit_for_square(new_square.value());
                current_square = new_square.value();
            },
        }
    }
    return board;
}


fn get_slide_direction_key(square: u8, dir: SlideDirection) -> u16 {
    return square as u16 + dir.get_hash_offset()
}


fn prepare_ray_bitboards() -> FxHashMap<u16, u64> {
    [
        SlideDirection::North,
        SlideDirection::NorthEast,
        SlideDirection::East,
        SlideDirection::SouthEast,
        SlideDirection::South,
        SlideDirection::SouthWest,
        SlideDirection::West,
        SlideDirection::NorthWest,
    ].iter().fold(Default::default(),|mut map, dir| {
        (0u8..=63u8).for_each(|s| {
            map.insert(get_slide_direction_key(s, *dir), generate_sliding_bitboard(s, *dir));
        });
        map
    })
}


fn generate_pawn_bitboard(square: u8, movement: PawnMovement) -> u64 {
    let mut board = 0u64;
    let mut current_square = square;
    for (col_shift, row_shift) in movement.get_movements() {
        for _ in 0u8..movement.get_max_distance(square) {
            match BoardSquare::from_value(current_square).apply_movement(col_shift, row_shift) {
                Err(_) => break,
                Ok(new_square) => {
                    board |= get_bit_for_square(new_square.value());
                    current_square = new_square.value();
                },
            }
        }
    }
    return board;
}


fn get_pawn_movement_key(square: u8, m: PawnMovement) -> u8 {
    return square + m.get_hash_offset()
}


fn prepare_pawn_bitboards() -> FxHashMap<u8, u64> {
    [
        PawnMovement::WhiteAdvance,
        PawnMovement::WhiteAttack,
        PawnMovement::BlackAdvance,
        PawnMovement::BlackAttack,
    ].iter().fold(Default::default(), |mut map, mov| {
        (0u8..=63u8).for_each(|s| {
            map.insert(get_pawn_movement_key(s, *mov), generate_pawn_bitboard(s, *mov));
        });
        map
    })
}


fn generate_bitboard_from_shifts(square: u8, shifts: Vec<(i8, i8)>) -> u64 {
    let bsquare = BoardSquare::from_value(square);
    return shifts.iter().fold(0u64, |board, (col_shift, row_shift)| {
        match bsquare.apply_movement(*col_shift, *row_shift) {
            Err(_) => board,
            Ok(new_square) => board | get_bit_for_square(new_square.value())
        }
    })
}


fn generate_knight_bitboard(square: u8) -> u64 {
    let shifts = Vec::from([
        (1i8, 2i8),
        (2i8, 1i8),
        (2i8, -1i8),
        (1i8, -2i8),
        (-1i8, -2i8),
        (-2i8, -1i8),
        (-2i8, 1i8),
        (-1i8, 2i8)
    ]);
    return generate_bitboard_from_shifts(square, shifts);
}


fn prepare_knight_bitboards() -> FxHashMap<u8, u64> {
    return (0u8..=63u8).fold(Default::default(), |mut map, s| {
        map.insert(s, generate_knight_bitboard(s));
        map
    })
}


fn generate_king_bitboard(square: u8) -> u64 {
    let shifts = Vec::from([
        (0i8, 1i8),
        (1i8, 1i8),
        (1i8, 0i8),
        (1i8, -1i8),
        (0i8, -1i8),
        (-1i8, -1i8),
        (-1i8, 0i8),
        (-1i8, 1i8),
    ]);
    return generate_bitboard_from_shifts(square, shifts);
}


fn prepare_king_bitboards() -> FxHashMap<u8, u64> {
    return (0u8..=63u8).fold(Default::default(), |mut map, s| {
        map.insert(s, generate_king_bitboard(s));
        map
    })
}


fn get_ray_bitboard(square: u8, dir: SlideDirection) -> u64 {
    return *RAY_BITBOARDS.get(&get_slide_direction_key(square, dir)).unwrap();
}

fn get_pawn_bitboard(square: u8, mov: PawnMovement) -> u64 {
    return *PAWN_BITBOARDS.get(&get_pawn_movement_key(square, mov)).unwrap();
}

fn get_knight_bitboard(square: u8) -> u64 {
    return *KNIGHT_BITBOARDS.get(&square).unwrap();
}

fn get_king_bitboard(square: u8) -> u64 {
    return *KING_BITBOARDS.get(&square).unwrap();
}

fn get_bitboard_for_slide_direction(square: u8, friendlies: u64, enemies: u64, dir: SlideDirection) -> u64 {
    let all_blockers = friendlies | enemies;
    let ray = get_ray_bitboard(square, dir);
    let blocks = ray & all_blockers;
    if blocks == 0u64 {
        return ray;
    }
    let first_block = match dir.is_positive() {
        true => blocks.trailing_zeros() as u8,
        false => 63 - blocks.leading_zeros() as u8,
    };
    let blocker_bit = get_bit_for_square(first_block);
    let blocked_squares = get_ray_bitboard(first_block, dir);
    let mut moves = ray ^ blocked_squares;
    if blocker_bit & enemies == 0 {
        moves &= !blocker_bit;
    }
    return moves;
}


fn get_bitboard_for_slide_directions<'a, I>(square: u8, friendlies: u64, enemies: u64, dirs: I) ->u64 where I: Iterator<Item=&'a SlideDirection> {
    return dirs.fold(0u64, |mut board, dir| {
        board |= get_bitboard_for_slide_direction(square, friendlies, enemies, *dir);
        board
    })    
}


fn get_bitboard_for_pawn_attacks(square: u8, enemies: u64, mov: PawnMovement, en_passant_target: u64) -> u64 {
    return get_pawn_bitboard(square, mov) & (enemies | en_passant_target);
}


fn get_bitboard_for_pawn_advance(square: u8, friendlies: u64, enemies: u64, mov: PawnMovement) -> u64 {
    let all_blockers = friendlies | enemies;
    let moves = get_pawn_bitboard(square, mov);
    let blocks = moves & all_blockers;
    if blocks == 0u64 {
        return moves;
    }
    let first_block = match mov.is_positive() {
        true => blocks.trailing_zeros() as u8,
        false => 63 - blocks.leading_zeros() as u8,
    };
    let blocked_squares = moves & (get_bit_for_square(first_block) | get_pawn_bitboard(first_block, mov));
    return moves ^ blocked_squares;
}


fn get_bitboard_for_pawn(square: u8, friendlies: u64, enemies: u64, color: Color, en_passant_target: u64) -> u64 {
    let advance = match color { Color::White => PawnMovement::WhiteAdvance, Color::Black => PawnMovement::BlackAdvance };
    let attack = match color { Color::White => PawnMovement::WhiteAttack, Color::Black => PawnMovement::BlackAttack };
    return get_bitboard_for_pawn_attacks(square, enemies, attack, en_passant_target) | get_bitboard_for_pawn_advance(square, friendlies, enemies, advance);
}


fn get_bitboard_for_knight(square: u8, friendlies: u64) -> u64 {
    return get_knight_bitboard(square) ^ friendlies
}


fn get_bitboard_for_king(square: u8, friendlies: u64) -> u64 {
    return get_king_bitboard(square) ^ friendlies
}


pub fn set_bit_at_square(board: u64, square: u8) -> u64 {
    return board | 2u64.pow(square as u32)
}


pub fn unset_bit_at_square(board: u64, square: u8) -> u64 {
    return board & !(2u64.pow(square as u32))
}


pub fn get_moves_bitboard_for_piece(square: u8, piece: Piece, friendlies: u64, enemies: u64, en_passant_target: u64) -> u64 {
    match piece.piece_type {
        PieceType::Pawn   => get_bitboard_for_pawn(square, friendlies, enemies, piece.color, en_passant_target),
        PieceType::Knight => get_bitboard_for_knight(square, friendlies),
        PieceType::Bishop => get_bitboard_for_slide_directions(square, friendlies, enemies, BISHOP_DIRS.iter()),
        PieceType::Rook   => get_bitboard_for_slide_directions(square, friendlies, enemies, ROOK_DIRS.iter()),
        PieceType::Queen  => get_bitboard_for_slide_directions(square, friendlies, enemies, QUEEN_DIRS.iter()),
        PieceType::King   => get_bitboard_for_king(square, friendlies),
    }
}