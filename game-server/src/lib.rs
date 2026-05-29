mod bitboard;
mod board;
mod square;

use wasm_bindgen::prelude::wasm_bindgen;
use worker::{DurableObject, Env, Request, Response, Result, State, durable_object};

use crate::board::Board;

#[durable_object]
pub struct GameServer {
    board: Board,
}

#[wasm_bindgen]
impl GameServer {
    #[wasm_bindgen]
    pub fn fen(&self) -> String {
        self.board.fen()
    }
}

impl DurableObject for GameServer {
    fn new(_: State, _: Env) -> Self {
        Self {
            board: Board::default(),
        }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        Response::ok("")
    }
}
