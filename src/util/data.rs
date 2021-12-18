lazy_static! {
    pub static ref ROW_1: FnvIndexSet<u8> = [  0u8,  1u8,  2u8,  3u8,  4u8,  5u8,  6u8,  7u8 ].into_iter().collect();
    pub static ref ROW_2: FnvIndexSet<u8> = [  8u8,  9u8, 10u8, 11u8, 12u8, 13u8, 14u8, 15u8 ].into_iter().collect();
    pub static ref ROW_3: FnvIndexSet<u8> = [ 16u8, 17u8, 18u8, 19u8, 20u8, 21u8, 22u8, 23u8 ].into_iter().collect();
    pub static ref ROW_4: FnvIndexSet<u8> = [ 24u8, 25u8, 26u8, 27u8, 28u8, 29u8, 30u8, 31u8 ].into_iter().collect();
    pub static ref ROW_5: FnvIndexSet<u8> = [ 32u8, 33u8, 34u8, 35u8, 36u8, 37u8, 38u8, 39u8 ].into_iter().collect();
    pub static ref ROW_6: FnvIndexSet<u8> = [ 40u8, 41u8, 42u8, 43u8, 44u8, 45u8, 46u8, 47u8 ].into_iter().collect();
    pub static ref ROW_7: FnvIndexSet<u8> = [ 48u8, 49u8, 50u8, 51u8, 52u8, 53u8, 54u8, 55u8 ].into_iter().collect();
    pub static ref ROW_8: FnvIndexSet<u8> = [ 56u8, 57u8, 58u8, 59u8, 60u8, 61u8, 62u8, 63u8 ].into_iter().collect();

    pub static ref COL_A: FnvIndexSet<u8> = [ 0u8,  8u8, 16u8, 24u8, 32u8, 40u8, 48u8, 56u8 ].into_iter().collect();
    pub static ref COL_B: FnvIndexSet<u8> = [ 1u8,  9u8, 17u8, 25u8, 33u8, 41u8, 49u8, 57u8 ].into_iter().collect();
    pub static ref COL_C: FnvIndexSet<u8> = [ 2u8, 10u8, 18u8, 26u8, 34u8, 42u8, 50u8, 58u8 ].into_iter().collect();
    pub static ref COL_D: FnvIndexSet<u8> = [ 3u8, 11u8, 19u8, 27u8, 35u8, 43u8, 51u8, 59u8 ].into_iter().collect();
    pub static ref COL_E: FnvIndexSet<u8> = [ 4u8, 12u8, 20u8, 28u8, 36u8, 44u8, 52u8, 60u8 ].into_iter().collect();
    pub static ref COL_F: FnvIndexSet<u8> = [ 5u8, 13u8, 21u8, 29u8, 37u8, 45u8, 53u8, 61u8 ].into_iter().collect();
    pub static ref COL_G: FnvIndexSet<u8> = [ 6u8, 14u8, 22u8, 30u8, 38u8, 46u8, 54u8, 62u8 ].into_iter().collect();
    pub static ref COL_H: FnvIndexSet<u8> = [ 7u8, 15u8, 23u8, 31u8, 39u8, 47u8, 55u8, 63u8 ].into_iter().collect();

    pub static ref DIAG_A7B8: FnvIndexSet<u8> = [                                     48u8, 57u8 ].into_iter().collect();
    pub static ref DIAG_A6C8: FnvIndexSet<u8> = [                               40u8, 49u8, 58u8 ].into_iter().collect();
    pub static ref DIAG_A5D8: FnvIndexSet<u8> = [                         32u8, 41u8, 50u8, 59u8 ].into_iter().collect();
    pub static ref DIAG_A4E8: FnvIndexSet<u8> = [                   24u8, 33u8, 42u8, 51u8, 60u8 ].into_iter().collect();
    pub static ref DIAG_A3F8: FnvIndexSet<u8> = [             16u8, 25u8, 34u8, 43u8, 52u8, 61u8 ].into_iter().collect();
    pub static ref DIAG_A2G8: FnvIndexSet<u8> = [        8u8, 17u8, 26u8, 35u8, 44u8, 53u8, 62u8 ].into_iter().collect();
    pub static ref DIAG_A1H8: FnvIndexSet<u8> = [  0u8,  9u8, 18u8, 27u8, 36u8, 45u8, 54u8, 63u8 ].into_iter().collect();
    pub static ref DIAG_B1H7: FnvIndexSet<u8> = [  1u8, 10u8, 19u8, 28u8, 37u8, 46u8, 55u8       ].into_iter().collect();
    pub static ref DIAG_C1H6: FnvIndexSet<u8> = [  2u8, 11u8, 20u8, 29u8, 38u8, 47u8             ].into_iter().collect();
    pub static ref DIAG_D1H5: FnvIndexSet<u8> = [  3u8, 12u8, 21u8, 30u8, 39u8                   ].into_iter().collect();
    pub static ref DIAG_E1H4: FnvIndexSet<u8> = [  4u8, 13u8, 22u8, 31u8                         ].into_iter().collect();
    pub static ref DIAG_F1H3: FnvIndexSet<u8> = [  5u8, 14u8, 23u8                               ].into_iter().collect();
    pub static ref DIAG_G1H2: FnvIndexSet<u8> = [  6u8, 15u8                                     ].into_iter().collect();

    pub static ref DIAG_H7G8: FnvIndexSet<u8> = [                                     55u8, 62u8 ].into_iter().collect();
    pub static ref DIAG_H6F8: FnvIndexSet<u8> = [                               47u8, 54u8, 61u8 ].into_iter().collect();
    pub static ref DIAG_H5E8: FnvIndexSet<u8> = [                         39u8, 46u8, 53u8, 60u8 ].into_iter().collect();
    pub static ref DIAG_H4D8: FnvIndexSet<u8> = [                   31u8, 38u8, 45u8, 52u8, 59u8 ].into_iter().collect();
    pub static ref DIAG_H3C8: FnvIndexSet<u8> = [             23u8, 30u8, 37u8, 44u8, 51u8, 58u8 ].into_iter().collect();
    pub static ref DIAG_H2B8: FnvIndexSet<u8> = [       15u8, 22u8, 29u8, 36u8, 43u8, 50u8, 57u8 ].into_iter().collect();
    pub static ref DIAG_H1A8: FnvIndexSet<u8> = [  7u8, 14u8, 21u8, 28u8, 35u8, 42u8, 49u8, 56u8 ].into_iter().collect();
    pub static ref DIAG_G1A7: FnvIndexSet<u8> = [  6u8, 13u8, 20u8, 27u8, 34u8, 41u8, 48u8       ].into_iter().collect();
    pub static ref DIAG_F1A6: FnvIndexSet<u8> = [  5u8, 12u8, 19u8, 26u8, 33u8, 40u8             ].into_iter().collect();
    pub static ref DIAG_E1A5: FnvIndexSet<u8> = [  4u8, 11u8, 18u8, 25u8, 32u8                   ].into_iter().collect();
    pub static ref DIAG_D1A4: FnvIndexSet<u8> = [  3u8, 10u8, 17u8, 24u8                         ].into_iter().collect();
    pub static ref DIAG_C1A3: FnvIndexSet<u8> = [  2u8,  9u8, 16u8                               ].into_iter().collect();
    pub static ref DIAG_B1A2: FnvIndexSet<u8> = [  1u8,  8u8                                     ].into_iter().collect();

    static ref GAME_PIECES: HashMap<GamePiece, Piece> = HashMap::from([
        (GamePiece::WhiteKing,        Piece::new(Color::White, PieceType::King  )), (GamePiece::WhiteQueen,        Piece::new(Color::White, PieceType::Queen )),
        (GamePiece::WhiteKingsRook,   Piece::new(Color::White, PieceType::Rook  )), (GamePiece::WhiteQueensRook,   Piece::new(Color::White, PieceType::Rook  )),
        (GamePiece::WhiteKingsBishop, Piece::new(Color::White, PieceType::Bishop)), (GamePiece::WhiteQueensBishop, Piece::new(Color::White, PieceType::Bishop)),
        (GamePiece::WhiteKingsKnight, Piece::new(Color::White, PieceType::Knight)), (GamePiece::WhiteQueensKnight, Piece::new(Color::White, PieceType::Knight)),
        (GamePiece::WhiteAPawn,       Piece::new(Color::White, PieceType::Pawn  )), (GamePiece::WhiteBPawn,        Piece::new(Color::White, PieceType::Pawn  )),
        (GamePiece::WhiteCPawn,       Piece::new(Color::White, PieceType::Pawn  )), (GamePiece::WhiteDPawn,        Piece::new(Color::White, PieceType::Pawn  )),
        (GamePiece::WhiteEPawn,       Piece::new(Color::White, PieceType::Pawn  )), (GamePiece::WhiteFPawn,        Piece::new(Color::White, PieceType::Pawn  )),
        (GamePiece::WhiteGPawn,       Piece::new(Color::White, PieceType::Pawn  )), (GamePiece::WhiteHPawn,        Piece::new(Color::White, PieceType::Pawn  )),
        (GamePiece::BlackKing,        Piece::new(Color::Black, PieceType::King  )), (GamePiece::BlackQueen,        Piece::new(Color::Black, PieceType::Queen )),
        (GamePiece::BlackKingsRook,   Piece::new(Color::Black, PieceType::Rook  )), (GamePiece::BlackQueensRook,   Piece::new(Color::Black, PieceType::Rook  )),
        (GamePiece::BlackKingsBishop, Piece::new(Color::Black, PieceType::Bishop)), (GamePiece::BlackQueensBishop, Piece::new(Color::Black, PieceType::Bishop)),
        (GamePiece::BlackKingsKnight, Piece::new(Color::Black, PieceType::Knight)), (GamePiece::BlackQueensKnight, Piece::new(Color::Black, PieceType::Knight)),
        (GamePiece::BlackAPawn,       Piece::new(Color::Black, PieceType::Pawn  )), (GamePiece::BlackBPawn,        Piece::new(Color::Black, PieceType::Pawn  )),
        (GamePiece::BlackCPawn,       Piece::new(Color::Black, PieceType::Pawn  )), (GamePiece::BlackDPawn,        Piece::new(Color::Black, PieceType::Pawn  )),
        (GamePiece::BlackEPawn,       Piece::new(Color::Black, PieceType::Pawn  )), (GamePiece::BlackFPawn,        Piece::new(Color::Black, PieceType::Pawn  )),
        (GamePiece::BlackGPawn,       Piece::new(Color::Black, PieceType::Pawn  )), (GamePiece::BlackHPawn,        Piece::new(Color::Black, PieceType::Pawn  )),
    ]);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GamePiece {
    WhiteKing,        WhiteQueen,        WhiteKingsRook,   WhiteQueensRook,
    WhiteKingsBishop, WhiteQueensBishop, WhiteKingsKnight, WhiteQueensKnight,
    WhiteAPawn,       WhiteBPawn,        WhiteCPawn,       WhiteDPawn,
    WhiteEPawn,       WhiteFPawn,        WhiteGPawn,       WhiteHPawn,
    BlackKing,        BlackQueen,        BlackKingsRook,   BlackQueensRook,
    BlackKingsBishop, BlackQueensBishop, BlackKingsKnight, BlackQueensKnight,
    BlackAPawn,       BlackBPawn,        BlackCPawn,       BlackDPawn,
    BlackEPawn,       BlackFPawn,        BlackGPawn,       BlackHPawn,
}