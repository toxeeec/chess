mod messages;
mod state;

use std::{cell::RefCell, str::FromStr};

pub use messages::SnapshotMessage;
use messages::{ClientMessage, ErrorMessage, MoveMessage, ServerMessage};
use state::GameServerState;
use wasm_bindgen::prelude::wasm_bindgen;
use worker::{
    DurableObject, Env, Request, Response, Result, State, WebSocket, WebSocketIncomingMessage,
    WebSocketPair, durable_object,
};

use crate::{
    game::{Game, Player},
    moves::Move,
    storage::{Storage, StoredGame},
};

const PLAYER_HEADER: &str = "Player-Color";

#[durable_object]
pub struct GameServer {
    state: RefCell<GameServerState>,
    storage: Storage,
    durable_state: State,
}

#[wasm_bindgen]
impl GameServer {
    #[wasm_bindgen]
    pub fn snapshot(&self) -> SnapshotMessage {
        self.state.borrow().snapshot_message()
    }
}

impl DurableObject for GameServer {
    fn new(durable_state: State, _: Env) -> Self {
        let storage = Storage::new(durable_state.storage());
        storage.init().unwrap();

        let stored_game = match storage.load().unwrap() {
            Some(stored_game) => stored_game,
            None => {
                let game = Game::default();
                storage.save(&game, 0).unwrap();
                StoredGame { game, revision: 0 }
            }
        };

        Self {
            state: RefCell::new(GameServerState::new(stored_game.game, stored_game.revision)),
            storage: Storage::new(durable_state.storage()),
            durable_state,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let player = match req.headers().get(PLAYER_HEADER)?.as_deref() {
            Some("white") => Player::White,
            Some("black") => Player::Black,
            _ => return Response::error("Forbidden", 403),
        };

        let pair = WebSocketPair::new()?;
        pair.server.serialize_attachment(player)?;
        self.durable_state.accept_web_socket(&pair.server);

        let state = self.state.borrow();
        pair.server
            .send(&ServerMessage::Snapshot(state.snapshot_message()))?;

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

        let Some(player) = ws.deserialize_attachment::<Player>()? else {
            ws.send(&ServerMessage::Error(ErrorMessage::InvalidPlayer))?;
            return Ok(());
        };

        let Ok(ClientMessage::Move(mve)) = serde_json::from_str::<ClientMessage>(&message) else {
            ws.send(&ServerMessage::Error(ErrorMessage::InvalidMessage))?;
            return Ok(());
        };
        let Ok(mve) = Move::from_str(&mve) else {
            ws.send(&ServerMessage::Error(ErrorMessage::InvalidMoveFormat))?;
            return Ok(());
        };

        let move_message = {
            let mut state = self.state.borrow_mut();
            match state.make_move(player, mve) {
                Ok(_) => {
                    self.storage.save(&state.game, state.revision)?;
                }
                Err(error) => {
                    ws.send(&ServerMessage::Error(error))?;
                    return Ok(());
                }
            }

            MoveMessage::new(mve, state.revision, state.game.turn, state.legal_moves())
        };

        let message = ServerMessage::Move(move_message);
        for socket in self.durable_state.get_websockets() {
            socket.send(&message)?;
        }

        Ok(())
    }
}
