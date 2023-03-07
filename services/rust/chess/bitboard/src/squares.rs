use crate::square::Square;

pub struct Squares {
    sq: Square,
}

pub const fn squares() -> Squares {
    Squares { sq: 0.into() }
}

impl const Iterator for Squares {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        let sq = self.sq;
        self.sq.0 += 1;
        if sq.0 < 64 {
            Some(sq)
        } else {
            None
        }
    }
}
