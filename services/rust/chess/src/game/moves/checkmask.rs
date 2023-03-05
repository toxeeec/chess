use crate::game::{
    board::Board,
    moves::{
        knight::KNIGHT_LOOKUP,
        magics::{bishop_moves, rook_moves},
        pins::CHECK_PATH,
    },
    piece::Piece,
};
use bitboard::{bb, for_each, shift::Direction, Bitboard};

static SQUARE_BEHIND: [[Bitboard; 64]; 64] = {
    let mut bbs = [[Bitboard::default(); 64]; 64];
    let mut king_sq = 0;
    while king_sq < 64 {
        let mut enemy_sq = 0;
        while enemy_sq < 64 {
            let dir = Direction::toward(king_sq, enemy_sq);
            if let Some(mut dir) = dir {
                if king_sq < enemy_sq {
                    dir = dir.opposite();
                }
                let bb = bb![king_sq].shifted_by(dir);
                bbs[king_sq as usize][enemy_sq as usize] = bb;
            }
            enemy_sq += 1;
        }
        king_sq += 1;
    }
    bbs
};

static PIN_PATH: [[Bitboard; 64]; 64] = {
    let mut bbs = CHECK_PATH;
    let mut i = 0;
    while i < 64 {
        let mut j = 0;
        while j < 64 {
            bbs[i][j] |= SQUARE_BEHIND[i][j];
            bbs[i][j] &= !bb![j as u32];
            j += 1
        }
        i += 1;
    }
    bbs
};

fn pawn_check(
    is_white: bool,
    mask: &mut Bitboard,
    board: &Board,
    king_sq: u32,
    king_ban: &mut Bitboard,
) {
    let left_pawns = board
        .get::<{ Piece::Pawn }>(!is_white)
        .shifted_backward_left(is_white);
    let right_pawns = board
        .get::<{ Piece::Pawn }>(!is_white)
        .shifted_backward_right(is_white);

    *king_ban |= left_pawns | right_pawns;
    if left_pawns.contains(king_sq) {
        *mask = bb![king_sq].shifted_forward_left(is_white);
    } else if right_pawns.contains(king_sq) {
        *mask = bb![king_sq].shifted_forward_right(is_white);
    }
}

fn knight_check(
    is_white: bool,
    mask: &mut Bitboard,
    board: &Board,
    king_sq: u32,
    king_ban: &mut Bitboard,
) {
    let mut bb = board.get::<{ Piece::Knight }>(!is_white);
    let mut sq;
    for_each!(bb, sq, {
        let attacks = KNIGHT_LOOKUP[sq as usize];
        *king_ban |= attacks;
        if attacks.contains(king_sq) {
            *mask = bb![sq];
        }
    });
}

fn bishop_check(
    is_white: bool,
    mask: &mut Bitboard,
    board: &Board,
    king_sq: u32,
    king_ban: &mut Bitboard,
) {
    let mut bb =
        board.get::<{ Piece::Bishop }>(!is_white) | board.get::<{ Piece::Queen }>(!is_white);
    let mut sq;
    for_each!(bb, sq, {
        let attacks = bishop_moves(sq, board.occ);
        *king_ban |= attacks;
        if attacks.contains(king_sq) {
            *king_ban |= PIN_PATH[king_sq as usize][sq as usize];
            *mask &= CHECK_PATH[king_sq as usize][sq as usize];
        }
    });
}

fn rook_check(
    is_white: bool,
    mask: &mut Bitboard,
    board: &Board,
    king_sq: u32,
    king_ban: &mut Bitboard,
) {
    let mut bb = board.get::<{ Piece::Rook }>(!is_white) | board.get::<{ Piece::Queen }>(!is_white);
    let mut sq;
    for_each!(bb, sq, {
        let attacks = rook_moves(sq, board.occ);
        *king_ban |= attacks;
        if attacks.contains(king_sq) {
            *king_ban |= PIN_PATH[king_sq as usize][sq as usize];
            *mask &= CHECK_PATH[king_sq as usize][sq as usize];
        }
    });
}

pub fn checkmask(is_white: bool, board: &Board, banned: &mut Bitboard) -> Bitboard {
    let king_sq = board.get::<{ Piece::King }>(is_white).lsb();
    let mut mask = !Bitboard::default();
    pawn_check(is_white, &mut mask, board, king_sq, banned);
    knight_check(is_white, &mut mask, board, king_sq, banned);
    // queen check is computed in both bishop and rook checks
    bishop_check(is_white, &mut mask, board, king_sq, banned);
    rook_check(is_white, &mut mask, board, king_sq, banned);
    mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0, 0 => Bitboard::default())]
    #[test_case(0, 3 => bb![1, 2, 3])]
    #[test_case(0, 24 => bb![8, 16, 24])]
    #[test_case(0, 27 => bb![9, 18, 27])]
    #[test_case(24, 3 => bb![3, 10, 17])]
    #[test_case(0, 25 => Bitboard::default())]
    fn check_path_tests(king_sq: usize, enemy_sq: usize) -> Bitboard {
        CHECK_PATH[king_sq][enemy_sq]
    }

    #[test_case(0, 0 => Bitboard::default())]
    #[test_case(0, 1 => Bitboard::default())]
    #[test_case(0, 9 => Bitboard::default())]
    #[test_case(63, 7 => Bitboard::default())]
    #[test_case(1, 2 => bb![0])]
    fn square_behind_tests(king_sq: usize, enemy_sq: usize) -> Bitboard {
        SQUARE_BEHIND[king_sq][enemy_sq]
    }

    #[test_case(0, 0 => Bitboard::default())]
    #[test_case(0, 3 => bb![1, 2])]
    #[test_case(8, 24 => bb![0, 16])]
    #[test_case(18, 27 => bb![9])]
    #[test_case(24, 3 => bb![10, 17])]
    #[test_case(0, 25 => Bitboard::default())]
    fn pin_path_tests(king_sq: usize, enemy_sq: usize) -> Bitboard {
        PIN_PATH[king_sq][enemy_sq]
    }
}
