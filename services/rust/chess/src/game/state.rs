use bitboard::{shift::Direction, square::Square};

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
                self.ep = Some(Square::new(mov.to()).shifted_by(dir));
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
                        4 => {
                            self.wk = false;
                            self.wq = false;
                        }
                        0 => self.wq = false,
                        7 => self.wk = false,
                        _ => (),
                    }
                } else {
                    match from {
                        60 => {
                            self.bk = false;
                            self.bq = false;
                        }
                        56 => self.bq = false,
                        63 => self.bk = false,
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
