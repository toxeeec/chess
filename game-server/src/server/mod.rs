mod messages;

pub use messages::SnapshotMessage;

use std::{
    cell::{Ref, RefCell, RefMut},
    str::FromStr,
};

use wasm_bindgen::prelude::wasm_bindgen;
use worker::{
    DurableObject, Env, Request, Response, Result, ScheduledTime, State, WebSocket,
    WebSocketIncomingMessage, WebSocketPair, durable_object,
    js_sys::{Date, Number},
};

use crate::{
    game::{Game, Player},
    game_state::{GameState, PlayerConnected, StateChange},
    game_storage::GameStorage,
    moves::Move,
    server::messages::{ClientMessage, ErrorMessage, MoveMessage, ServerMessage},
};

const PLAYER_HEADER: &str = "Player-Color";

#[durable_object]
pub struct GameServer {
    state: RefCell<Option<GameState>>,
    storage: GameStorage,
    durable_state: State,
}

#[wasm_bindgen]
impl GameServer {
    #[wasm_bindgen]
    pub async fn init(
        &self,
        join_timeout_ms: i32,
        first_move_timeout_ms: i32,
        disconnect_timeout_ms: i32,
    ) -> Result<()> {
        if self.state.borrow().is_some() {
            return Ok(());
        }

        self.create_game(
            join_timeout_ms,
            first_move_timeout_ms,
            disconnect_timeout_ms,
        )?;
        self.schedule_next_alarm().await?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn snapshot(&self) -> SnapshotMessage {
        SnapshotMessage::from(&*self.state().expect("game is not initialized"))
    }
}

impl DurableObject for GameServer {
    fn new(durable_state: State, _: Env) -> Self {
        let storage = GameStorage::new(durable_state.storage());
        storage.init().unwrap();

        let state = storage.load().unwrap();

        Self {
            state: RefCell::new(state),
            storage,
            durable_state,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let player = match req.headers().get(PLAYER_HEADER)?.as_deref() {
            Some("white") => Player::White,
            Some("black") => Player::Black,
            _ => return Response::error("Forbidden", 403),
        };

        let snapshot = match self.state() {
            Ok(state) => SnapshotMessage::from(&*state),
            Err(err) => return Response::error(err.to_string(), 409),
        };

        let pair = WebSocketPair::new()?;
        pair.server.serialize_attachment(player)?;
        self.durable_state.accept_web_socket(&pair.server);
        pair.server.send(&ServerMessage::Snapshot(snapshot))?;

        self.handle_player_connected(player).await?;
        Response::from_websocket(pair.client)
    }

    async fn alarm(&self) -> Result<Response> {
        {
            let mut state = self.state_mut()?;
            let change = state.process_due_event(Date::now() as i64);
            self.handle_state_change(&state, change)?;
        };

        self.schedule_next_alarm().await?;
        Response::ok("ok")
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
            let mut state = match self.state_mut() {
                Ok(state) => state,
                Err(error) => {
                    ws.close(Some(1011), Some("game is not initialized"))?;
                    return Err(error);
                }
            };

            if let Err(error) = state.make_move(player, mve) {
                ws.send(&ServerMessage::Error(error.into()))?;
                return Ok(());
            }

            self.storage.save(&state)?;
            MoveMessage::new(mve, state.revision, &state.game)
        };

        let message = ServerMessage::Move(move_message);
        for socket in self.durable_state.get_websockets() {
            socket.send(&message)?;
        }

        self.schedule_next_alarm().await?;
        Ok(())
    }

    async fn websocket_close(
        &self,
        ws: WebSocket,
        _code: usize,
        _reason: String,
        _was_clean: bool,
    ) -> Result<()> {
        let Some(player) = ws.deserialize_attachment::<Player>()? else {
            return Ok(());
        };

        self.handle_player_disconnected(player).await?;
        Ok(())
    }
}

impl GameServer {
    fn create_game(
        &self,
        join_timeout_ms: i32,
        first_move_timeout_ms: i32,
        disconnect_timeout_ms: i32,
    ) -> Result<()> {
        let game_state = self.storage.create_game(
            Game::default(),
            join_timeout_ms,
            first_move_timeout_ms,
            disconnect_timeout_ms,
        )?;
        self.state.replace(Some(game_state));

        Ok(())
    }

    async fn handle_player_connected(&self, player: Player) -> Result<()> {
        let is_white_connected = self.is_player_connected(Player::White)?;
        let is_black_connected = self.is_player_connected(Player::Black)?;

        {
            let mut state = self.state_mut()?;
            let change = state.player_connected(PlayerConnected {
                player,
                now: Date::now() as i64,
                is_white_connected,
                is_black_connected,
            });
            self.handle_state_change(&state, change)?;
        };

        self.schedule_next_alarm().await?;
        Ok(())
    }

    async fn handle_player_disconnected(&self, player: Player) -> Result<()> {
        if self.is_player_connected(player)? {
            return Ok(());
        }

        {
            let mut state = self.state_mut()?;
            let change = state.player_disconnected(player, Date::now() as i64);
            self.handle_state_change(&state, change)?;
        }

        self.schedule_next_alarm().await?;
        Ok(())
    }

    fn handle_state_change(&self, state: &GameState, change: StateChange) -> Result<()> {
        match change {
            StateChange::LifecycleChanged => {
                self.storage.save(state)?;

                let message = ServerMessage::Status(state.into());
                for socket in self.durable_state.get_websockets() {
                    socket.send(&message)?;
                }
            }
            StateChange::Updated => self.storage.save(state)?,
            StateChange::Unchanged => {}
        }

        Ok(())
    }

    async fn schedule_next_alarm(&self) -> Result<()> {
        let next_alarm = self.state()?.next_event_at();

        match next_alarm {
            Some(next_alarm) => {
                let timestamp = Number::from(next_alarm as f64);
                self.durable_state
                    .storage()
                    .set_alarm(ScheduledTime::new(Date::new(&timestamp)))
                    .await?;
            }
            None => self.durable_state.storage().delete_alarm().await?,
        }

        Ok(())
    }

    fn is_player_connected(&self, player: Player) -> Result<bool> {
        for socket in self.durable_state.get_websockets() {
            if socket.deserialize_attachment::<Player>()? == Some(player) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn state(&self) -> Result<Ref<'_, GameState>> {
        Ref::filter_map(self.state.borrow(), Option::as_ref)
            .map_err(|_| worker::Error::RustError("game is not initialized".to_string()))
    }

    fn state_mut(&self) -> Result<RefMut<'_, GameState>> {
        RefMut::filter_map(self.state.borrow_mut(), Option::as_mut)
            .map_err(|_| worker::Error::RustError("game is not initialized".to_string()))
    }
}
