use crate::game::{
    board::Board,
    moves::magics::{bishop_moves, rook_moves},
    piece::Piece,
};
use bitboard::{bb, for_each, shift::Direction, Bitboard};

#[derive(Debug)]
pub struct Pins {
    pub hv: Bitboard,
    pub diag: Bitboard,
}

impl Pins {
    pub fn new(is_white: bool, board: &Board) -> Self {
        let hv = hv_pins(is_white, board);
        let diag = diag_pins(is_white, board);
        Self { hv, diag }
    }
}

fn hv_pins(is_white: bool, board: &Board) -> Bitboard {
    let own = board.own(is_white);
    let king_bb = board.get::<{ Piece::King }>(is_white);
    let king_sq = king_bb.lsb();
    let mut bb = board.get::<{ Piece::Rook }>(!is_white) | board.get::<{ Piece::Queen }>(!is_white);
    let mut pins = Bitboard::default();

    let mut sq;
    for_each!(bb, sq, {
        let blockers = rook_moves(sq, own) & own;
        let pinned = rook_moves(sq, board.occ ^ blockers);
        if !(pinned & king_bb).is_empty() {
            pins |= CHECK_PATH[king_sq as usize][sq as usize];
        };
    });
    pins
}

fn diag_pins(is_white: bool, board: &Board) -> Bitboard {
    let own = board.own(is_white);
    let king_bb = board.get::<{ Piece::King }>(is_white);
    let king_sq = king_bb.lsb();
    let mut bb =
        board.get::<{ Piece::Bishop }>(!is_white) | board.get::<{ Piece::Queen }>(!is_white);
    let mut pins = Bitboard::default();

    let mut sq;
    for_each!(bb, sq, {
        let blockers = bishop_moves(sq, own) & own;
        let pinned = bishop_moves(sq, board.occ ^ blockers);
        if !(pinned & king_bb).is_empty() {
            pins |= CHECK_PATH[king_sq as usize][sq as usize];
        };
    });
    pins
}

pub(crate) static CHECK_PATH: [[Bitboard; 64]; 64] = {
    let mut bbs = [[Bitboard::default(); 64]; 64];
    let mut king_sq = 0;
    while king_sq < 64 {
        let mut enemy_sq = 0;
        while enemy_sq < 64 {
            let dir = Direction::toward(king_sq, enemy_sq);
            if let Some(dir) = dir {
                let mut sq = king_sq;
                let mut bb = Bitboard::default();
                while sq != enemy_sq {
                    if king_sq < enemy_sq {
                        sq += dir as u32;
                    } else {
                        sq -= dir as u32;
                    }
                    bb |= bb![sq];
                }
                bbs[king_sq as usize][enemy_sq as usize] = bb;
            }
            enemy_sq += 1;
        }
        king_sq += 1;
    }
    bbs
};

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
}
