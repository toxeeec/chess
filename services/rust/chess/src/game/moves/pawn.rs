use super::{pins::Pins, Move};
use crate::game::{
    board::Board,
    moves::{magics::rook_moves, Type},
    piece::Piece,
    state::State,
};
use bitboard::{
    bb, for_each, shift::Direction, square::Square, Bitboard, RANK_1, RANK_3, RANK_6, RANK_8,
};

const fn last_rank(is_white: bool) -> Bitboard {
    if is_white {
        RANK_8
    } else {
        RANK_1
    }
}

const fn third_rank(is_white: bool) -> Bitboard {
    if is_white {
        RANK_3
    } else {
        RANK_6
    }
}

const fn from<T: ~const Into<u32>>(is_white: bool, to: Square, dir: T) -> Square {
    if is_white {
        (to.0 - dir.into()).into()
    } else {
        (to.0 + dir.into()).into()
    }
}

fn single_pushes(is_white: bool, mut bb: Bitboard, list: &mut Vec<Move>) {
    let mut to;
    for_each!(bb, to, {
        let from = from(is_white, to, Direction::North);
        list.push(Move::new(from, to, Type::Quiet));
    });
}

fn promotions(is_white: bool, mut bb: Bitboard, list: &mut Vec<Move>) {
    let mut to;
    for_each!(bb, to, {
        let from = from(is_white, to, Direction::North);
        list.push(Move::new(from, to, Type::KnightPromotion));
        list.push(Move::new(from, to, Type::BishopPromotion));
        list.push(Move::new(from, to, Type::RookPromotion));
        list.push(Move::new(from, to, Type::QueenPromotion));
    });
}

fn double_pushes(is_white: bool, mut bb: Bitboard, list: &mut Vec<Move>) {
    let mut to;
    for_each!(bb, to, {
        let from = from(is_white, to, Direction::North as u32 * 2);
        list.push(Move::new(from, to, Type::DoublePush));
    });
}

fn captures<const IS_LEFT: bool>(is_white: bool, mut bb: Bitboard, list: &mut Vec<Move>) {
    let dir = if IS_LEFT {
        Direction::NorthWest
    } else {
        Direction::NorthEast
    };
    let mut to;
    for_each!(bb, to, {
        let from = from(is_white, to, dir);
        list.push(Move::new(from, to, Type::Capture));
    });
}

fn promotion_captures<const IS_LEFT: bool>(is_white: bool, mut bb: Bitboard, list: &mut Vec<Move>) {
    let dir = if IS_LEFT {
        Direction::NorthWest
    } else {
        Direction::NorthEast
    };
    let mut to;
    for_each!(bb, to, {
        let from = from(is_white, to, dir);
        list.push(Move::new(from, to, Type::KnightPromotionCapture));
        list.push(Move::new(from, to, Type::BishopPromotionCapture));
        list.push(Move::new(from, to, Type::RookPromotionCapture));
        list.push(Move::new(from, to, Type::QueenPromotionCapture));
    });
}

#[inline(always)]
pub fn pawn(board: &Board, state: State, list: &mut Vec<Move>, pins: &Pins, checkmask: Bitboard) {
    let bb = board.get::<{ Piece::Pawn }>(state.white);
    let empty = !board.occ;
    let not_diag_pinned = bb & !pins.diag;

    let pinned = not_diag_pinned.shifted_forward(state.white) & pins.hv;
    let mut pushed = (not_diag_pinned & !pins.hv).shifted_forward(state.white);
    pushed |= pinned;
    pushed &= empty;
    let mut double_pushed = pushed;
    pushed &= checkmask;
    single_pushes(state.white, pushed & !last_rank(state.white), list);
    promotions(state.white, pushed & last_rank(state.white), list);

    double_pushed &= third_rank(state.white);
    double_pushed = double_pushed.shifted_forward(state.white);
    double_pushed &= empty & checkmask;
    double_pushes(state.white, double_pushed, list);

    let not_hv_pinned = bb & !pins.hv;
    let mut pinned = not_hv_pinned.shifted_forward_left(state.white) & pins.diag;
    let mut shifted = (not_hv_pinned & !pins.diag).shifted_forward_left(state.white) | pinned;
    shifted &= board.enemy(state.white) & checkmask;

    captures::<true>(state.white, shifted & !last_rank(state.white), list);
    promotion_captures::<true>(state.white, shifted & last_rank(state.white), list);

    pinned = not_hv_pinned.shifted_forward_right(state.white) & pins.diag;
    shifted = (not_hv_pinned & !pins.diag).shifted_forward_right(state.white) | pinned;
    shifted &= board.enemy(state.white) & checkmask;

    captures::<false>(state.white, shifted & !last_rank(state.white), list);
    promotion_captures::<false>(state.white, shifted & last_rank(state.white), list);

    // TODO: refactor
    let Some(ep_sq) = state.ep else { return };
    let ep_bb = Bitboard::from(ep_sq);
    let mut bb = (ep_bb.shifted_backward_left(state.white)
        | ep_bb.shifted_backward_right(state.white))
        & not_hv_pinned;

    let ep_pawn = from(state.white, ep_sq, Direction::North);
    let mut from;
    for_each!(bb, from, {
        let mut queen_or_rook = board.get::<{ Piece::Queen }>(!state.white)
            | board.get::<{ Piece::Rook }>(!state.white);
        let king_bb = board.get::<{ Piece::King }>(state.white);

        // https://lichess.org/editor/8/8/8/kq1pP1K1/8/8/8/8_w_-_d6_0_1
        let occ = board.occ & !(bb![from.0, ep_pawn.0]);
        let mut qr_sq;
        for_each!(queen_or_rook, qr_sq, {
            if rook_moves(qr_sq, occ) & king_bb != Bitboard::default() {
                return;
            }
        });

        let is_pinned = pins.diag.contains(from);
        let is_ep_square_pinned = pins.diag.contains(ep_sq);
        if !is_pinned || is_ep_square_pinned {
            list.push(Move::new(from, ep_sq, Type::EnPassant));
        }
    });
}
