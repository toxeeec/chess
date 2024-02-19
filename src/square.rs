use crate::bitboard::Direction;

#[derive(Clone, Copy, Debug)]
pub(super) struct Square(pub(super) u32);

impl Square {
    pub(super) const ZERO: Self = Square(0);

    #[inline(always)]
    pub(super) const fn new(square: u32) -> Self {
        debug_assert!(square < 64);
        Self(square)
    }

    #[inline(always)]
    pub(super) const fn shifted(self, dir: Direction) -> Option<Self> {
        let sq = Self(self.0.wrapping_add(dir as u32));
        if sq.0 < 64 {
            Some(sq)
        } else {
            None
        }
    }

    #[inline(always)]
    pub(super) const fn file(self) -> u32 {
        self.0 % 8
    }

    #[inline(always)]
    pub(super) const fn rank(self) -> u32 {
        self.0 / 8
    }

    #[inline(always)]
    pub(super) const fn is_valid(self) -> bool {
        self.0 < 64
    }

    #[inline(always)]
    pub(super) const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u32> for Square {
    #[inline(always)]
    fn from(square: u32) -> Self {
        Self(square)
    }
}
