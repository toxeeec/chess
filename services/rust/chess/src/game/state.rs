use bitboard::{
    shift::Direction,
    square::{
        Square, BLACK_KING_SQ, BLACK_LEFT_ROOK_SQ, BLACK_RIGHT_ROOK_SQ, WHITE_KING_SQ,
        WHITE_LEFT_ROOK_SQ, WHITE_RIGHT_ROOK_SQ,
    },
};

use super::moves::{Move, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub white: bool,
    pub wk: bool,
    pub wq: bool,
    pub bk: bool,
    pub bq: bool,
    pub ep: Option<Square>,
}

impl State {
    pub fn update(&mut self, mov: Move) {
        self.ep = None;
        match mov.typ() {
            Type::DoublePush => {
                let dir = if self.white {
                    Direction::South
                } else {
                    Direction::North
                };
                self.ep = mov.to().shifted_by(dir);
            }
            Type::KingCastle | Type::QueenCastle => {
                if self.white {
                    self.wk = false;
                    self.wq = false;
                } else {
                    self.bk = false;
                    self.bq = false;
                }
            }
            _ => {
                let from = mov.from();
                if self.white {
                    match from {
                        WHITE_KING_SQ => {
                            self.wk = false;
                            self.wq = false;
                        }
                        WHITE_LEFT_ROOK_SQ => self.wq = false,
                        WHITE_RIGHT_ROOK_SQ => self.wk = false,
                        _ => (),
                    }
                } else {
                    match from {
                        BLACK_KING_SQ => {
                            self.bk = false;
                            self.bq = false;
                        }
                        BLACK_LEFT_ROOK_SQ => self.bq = false,
                        BLACK_RIGHT_ROOK_SQ => self.bk = false,
                        _ => (),
                    }
                }
            }
        }
        self.white = !self.white;
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            white: true,
            wk: true,
            wq: true,
            bk: true,
            bq: true,
            ep: None,
        }
    }
}
