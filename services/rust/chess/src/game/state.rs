use bitboard::shift::Direction;

use super::moves::{Move, Type};

#[derive(Debug, Clone, Copy)]
pub struct State {
    pub white: bool,
    pub wk: bool,
    pub wq: bool,
    pub bk: bool,
    pub bq: bool,
    pub ep: Option<u32>,
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
                self.ep = Some(dir.shift(mov.to()));
            }
            Type::KingCastle => {
                if self.white {
                    self.wk = false
                } else {
                    self.bk = false
                }
            }
            Type::QueenCastle => {
                if self.white {
                    self.wq = false
                } else {
                    self.bq = false
                }
            }
            _ => (),
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
