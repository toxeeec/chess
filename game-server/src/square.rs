#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Square(pub(super) u32);

impl Square {
    pub(super) const fn new(square: u32) -> Self {
        debug_assert!(square < 64);
        Self(square)
    }
}

impl From<u32> for Square {
    fn from(square: u32) -> Self {
        Self(square)
    }
}
