#[derive(Debug, Clone, Copy)]
pub struct State {
    pub white: bool,
    pub wk: bool,
    pub wq: bool,
    pub bk: bool,
    pub bq: bool,
    pub ep: Option<u32>,
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
