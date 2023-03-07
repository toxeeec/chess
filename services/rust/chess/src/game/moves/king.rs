use super::Move;
use crate::game::{board::Board, moves::Type, piece::Piece, state::State};
use bitboard::{
    bb, for_each,
    shift::Direction,
    square::{
        BLACK_KING_KING_CASTLE_SQ, BLACK_KING_QUEEN_CASTLE_SQ, BLACK_KING_SQ,
        WHITE_KING_KING_CASTLE_SQ, WHITE_KING_QUEEN_CASTLE_SQ, WHITE_KING_SQ,
    },
    Bitboard,
};

pub fn king(board: &Board, state: State, list: &mut Vec<Move>, banned: Bitboard) {
    let from = board.get::<{ Piece::King }>(state.white).lsb();
    let enemy = board.enemy(state.white);
    let enemy_or_empty = board.enemy_or_empty(state.white);
    let mut moves = KING_LOOKUP[from.0 as usize] & enemy_or_empty & !banned;
    let mut to;
    for_each!(moves, to, {
        let typ = if enemy.contains(to) {
            Type::Capture
        } else {
            Type::Quiet
        };
        let m = Move::new(from, to, typ);
        list.push(m);
    });

    king_castle(board, state, banned, list);
    queen_castle(board, state, banned, list);
}

pub(crate) const KING_LOOKUP: [Bitboard; 64] = {
    let mut bbs = [Bitboard::default(); 64];
    let mut i = 0;
    while i < 64 {
        let bb = bb![i];
        bbs[i as usize] = bb.shifted::<{ Direction::North }>()
            | bb.shifted::<{ Direction::NorthEast }>()
            | bb.shifted::<{ Direction::East }>()
            | bb.shifted::<{ Direction::SouthEast }>()
            | bb.shifted::<{ Direction::South }>()
            | bb.shifted::<{ Direction::SouthWest }>()
            | bb.shifted::<{ Direction::West }>()
            | bb.shifted::<{ Direction::NorthWest }>();
        i += 1;
    }
    bbs
};

fn right_rook_present(is_white: bool, bb: Bitboard) -> bool {
    const WK_ROOK: Bitboard = bb![7];
    const BK_ROOK: Bitboard = bb![63];
    let rook = if is_white { WK_ROOK } else { BK_ROOK };
    !(rook & bb).is_empty()
}

fn left_rook_present(is_white: bool, bb: Bitboard) -> bool {
    const WQ_ROOK: Bitboard = bb![0];
    const BQ_ROOK: Bitboard = bb![56];
    let rook = if is_white { WQ_ROOK } else { BQ_ROOK };
    !(rook & bb).is_empty()
}

fn can_king_castle(is_white: bool, board: &Board, banned: Bitboard) -> bool {
    const WK_PATH: Bitboard = bb![4, 5, 6];
    const WK_BETWEEN: Bitboard = bb![5, 6];
    const BK_PATH: Bitboard = bb![60, 61, 62];
    const BK_BETWEEN: Bitboard = bb![61, 62];
    let between = if is_white { WK_BETWEEN } else { BK_BETWEEN };
    let path = if is_white { WK_PATH } else { BK_PATH };

    if !(between & board.occ).is_empty() || !(path & banned).is_empty() {
        false
    } else {
        right_rook_present(is_white, board.get::<{ Piece::Rook }>(is_white))
    }
}

fn can_queen_castle(is_white: bool, board: &Board, banned: Bitboard) -> bool {
    const WQ_PATH: Bitboard = bb![2, 3, 4];
    const WQ_BETWEEN: Bitboard = bb![1, 2, 3];
    const BQ_PATH: Bitboard = bb![58, 59, 60];
    const BQ_BETWEEN: Bitboard = bb![57, 58, 59];
    let between = if is_white { WQ_BETWEEN } else { BQ_BETWEEN };
    let path = if is_white { WQ_PATH } else { BQ_PATH };

    if !(between & board.occ).is_empty() || !(path & banned).is_empty() {
        false
    } else {
        left_rook_present(is_white, board.get::<{ Piece::Rook }>(is_white))
    }
}

fn king_castle(board: &Board, state: State, banned: Bitboard, list: &mut Vec<Move>) {
    if (state.white && !state.wk) || (!state.white && !state.bk) {
        return;
    }
    if !can_king_castle(state.white, board, banned) {
        return;
    }
    let from = if state.white {
        WHITE_KING_SQ
    } else {
        BLACK_KING_SQ
    };
    let to = if state.white {
        WHITE_KING_KING_CASTLE_SQ
    } else {
        BLACK_KING_KING_CASTLE_SQ
    };
    list.push(Move::new(from, to, Type::KingCastle));
}

fn queen_castle(board: &Board, state: State, banned: Bitboard, list: &mut Vec<Move>) {
    if (state.white && !state.wq) || (!state.white && !state.bq) {
        return;
    }
    if !can_queen_castle(state.white, board, banned) {
        return;
    }
    let from = if state.white {
        WHITE_KING_SQ
    } else {
        BLACK_KING_SQ
    };
    let to = if state.white {
        WHITE_KING_QUEEN_CASTLE_SQ
    } else {
        BLACK_KING_QUEEN_CASTLE_SQ
    };
    list.push(Move::new(from, to, Type::QueenCastle));
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitboard::bb;
    use test_case::test_case;

    #[test_case(0 => bb![1, 8, 9])]
    #[test_case(9 => bb![0, 1, 2, 8, 10, 16, 17, 18])]
    fn king_lookup_tests(sq: usize) -> Bitboard {
        KING_LOOKUP[sq]
    }
}
