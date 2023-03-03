use bitboard::Bitboard;

#[derive(Debug)]
pub struct Pins {
    pub hv: Bitboard,
    pub diag: Bitboard,
}
