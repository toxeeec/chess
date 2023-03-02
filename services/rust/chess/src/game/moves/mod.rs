mod knight;
mod pins;

#[repr(u32)]
pub enum Type {
    Quiet,
    Capture,
}

// From   | To     | Type
// xxxxxx | xxxxxx | xxxx
// 15-10  | 9-4    | 3-0
struct Move(u16);

impl Move {
    fn new(from: u32, to: u32, typ: Type) -> Self {
        Self(((from as u16) << 6) | ((to as u16) << 4) | (typ as u16))
    }
}
