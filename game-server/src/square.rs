#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Square(pub(super) u32);

impl Square {
    pub(super) const fn new(square: u32) -> Self {
        debug_assert!(square < 64);
        Self(square)
    }

    pub(super) const fn backward<const IS_WHITE: bool>(self, n: u32) -> Self {
        if IS_WHITE {
            Square::new(self.0 - n * 8)
        } else {
            Square::new(self.0 + n * 8)
        }
    }
}
