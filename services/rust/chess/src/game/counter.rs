use super::{
    board::Board,
    moves::{Move, Type},
    piece::Piece,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Counter {
    pub half: u32,
    pub full: u32,
}

impl Counter {
    pub fn update(&mut self, mov: Move, is_white: bool, board: &Board) {
        let typ = mov.typ();
        match typ {
            Type::Quiet => {
                if board.get::<{ Piece::Pawn }>(is_white).contains(mov.from()) {
                    self.half = 0
                }
            }
            Type::DoublePush | Type::Capture => self.half = 0,
            _ => self.half += 1,
        }
        if !is_white {
            self.full += 1;
        }
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self { half: 0, full: 1 }
    }
}
