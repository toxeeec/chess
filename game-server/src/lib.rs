mod bitboard;
mod board;
mod game;
mod moves;
mod pawn;
mod square;

use std::fmt::Write;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;
use worker::{
    DurableObject, Env, Request, Response, Result, State, WebSocket, WebSocketIncomingMessage,
    WebSocketPair, durable_object,
};

use crate::game::Game;

#[durable_object]
pub struct GameServer {
    game: Game,
    cached_state: GameState,
    ctx: State,
}

#[derive(Clone, Serialize)]
#[wasm_bindgen(getter_with_clone)]
pub struct GameState {
    pub fen: String,
    pub moves: String,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "state")]
enum ServerMessage<'a> {
    #[serde(rename = "snapshot")]
    Snapshot(&'a GameState),
}

#[wasm_bindgen]
impl GameServer {
    #[wasm_bindgen]
    pub fn state(&self) -> GameState {
        self.cached_state.clone()
    }
}

impl From<&Game> for GameState {
    fn from(game: &Game) -> Self {
        let mut moves = String::with_capacity(game.moves.len() * 5);

        let mut iter = game.moves.iter();
        if let Some(first) = iter.next() {
            write!(&mut moves, "{}", first).unwrap();
            for mve in iter {
                write!(&mut moves, " {}", mve).unwrap();
            }
        }

        Self {
            fen: game.board.fen(),
            moves,
        }
    }
}

impl DurableObject for GameServer {
    fn new(ctx: State, _: Env) -> Self {
        let game = Game::default();
        let cached_state = GameState::from(&game);

        Self {
            game,
            ctx,
            cached_state,
        }
    }

    async fn fetch(&self, _req: Request) -> Result<Response> {
        let pair = WebSocketPair::new()?;
        self.ctx.accept_web_socket(&pair.server);

        pair.server
            .send(&ServerMessage::Snapshot(&self.cached_state))?;

        Response::from_websocket(pair.client)
    }

    async fn websocket_message(
        &self,
        ws: WebSocket,
        message: WebSocketIncomingMessage,
    ) -> Result<()> {
        let WebSocketIncomingMessage::String(message) = message else {
            ws.close(Some(1003), Some("binary messages are not supported"))?;
            return Ok(());
        };

        ws.send_with_str(format!("[Durable Object] message: {message}"))
    }
}
