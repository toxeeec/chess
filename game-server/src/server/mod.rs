mod messages;
mod state;
mod storage;

pub use messages::SnapshotMessage;

use std::{
    cell::{Ref, RefCell, RefMut},
    panic::AssertUnwindSafe,
};

use wasm_bindgen::prelude::wasm_bindgen;
use worker::{
    DurableObject, Env, Request, Response, Result, ScheduledTime, State, WebSocket,
    WebSocketIncomingMessage, WebSocketPair, durable_object,
    js_sys::{Date, Number},
};

use crate::{game::Game, state::Color};

use self::{
    messages::{ClientMessage, ErrorMessage, MoveMessage, ServerMessage, StatusMessage},
    state::{GameState, PlayerConnected, StateChange},
    storage::GameStorage,
};

const COLOR_HEADER: &str = "Player-Color";

#[durable_object]
pub struct GameServer {
    state: AssertUnwindSafe<RefCell<Option<GameState>>>,
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
        time_control_ms: i32,
    ) -> Result<()> {
        if self.state.borrow().is_some() {
            return Ok(());
        }

        self.create_game(
            join_timeout_ms,
            first_move_timeout_ms,
            disconnect_timeout_ms,
            time_control_ms,
        )?;
        self.schedule_next_alarm().await?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn snapshot(&self) -> SnapshotMessage {
        SnapshotMessage::new(
            &self.state().expect("game is not initialized"),
            Date::now() as i64,
        )
    }
}

impl DurableObject for GameServer {
    fn new(durable_state: State, _: Env) -> Self {
        let storage = GameStorage::new(durable_state.storage());
        storage.init().unwrap();

        let state = storage.load().unwrap();

        Self {
            state: AssertUnwindSafe(RefCell::new(state)),
            storage,
            durable_state,
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let color = match req.headers().get(COLOR_HEADER)?.as_deref() {
            Some("white") => Color::White,
            Some("black") => Color::Black,
            _ => return Response::error("Forbidden", 403),
        };

        let now = Date::now() as i64;
        let snapshot = match self.state() {
            Ok(state) => SnapshotMessage::new(&state, now),
            Err(err) => return Response::error(err.to_string(), 409),
        };

        let pair = WebSocketPair::new()?;
        pair.server.serialize_attachment(color)?;
        self.durable_state.accept_web_socket(&pair.server);
        pair.server.send(&ServerMessage::Snapshot(snapshot))?;

        self.handle_player_connected(color).await?;
        Response::from_websocket(pair.client)
    }

    async fn alarm(&self) -> Result<Response> {
        {
            let now = Date::now() as i64;
            let mut state = self.state_mut()?;
            let change = state.process_due_timeout(now);
            self.handle_state_change(&state, change, now)?;
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

        let Some(color) = ws.deserialize_attachment::<Color>()? else {
            ws.send(&ServerMessage::Error(ErrorMessage::InvalidPlayer))?;
            return Ok(());
        };

        let Ok(ClientMessage::Move(mve)) = serde_json::from_str::<ClientMessage>(&message) else {
            ws.send(&ServerMessage::Error(ErrorMessage::InvalidMessage))?;
            return Ok(());
        };
        let Ok(mve) = mve.parse() else {
            ws.send(&ServerMessage::Error(ErrorMessage::InvalidMoveFormat))?;
            return Ok(());
        };

        let now = Date::now() as i64;
        let move_message = {
            let mut state = self.state_mut()?;

            let due_change = state.process_due_timeout(now);
            if due_change != StateChange::Unchanged {
                self.handle_state_change(&state, due_change, now)?;
                return Ok(());
            }

            if let Err(error) = state.make_move(color, mve, now) {
                ws.send(&ServerMessage::Error(error.into()))?;
                return Ok(());
            }

            self.storage.save(&state)?;
            MoveMessage::new(mve, &state, now)
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
        let Some(color) = ws.deserialize_attachment::<Color>()? else {
            return Ok(());
        };

        self.handle_player_disconnected(color).await?;
        Ok(())
    }
}

impl GameServer {
    fn create_game(
        &self,
        join_timeout_ms: i32,
        first_move_timeout_ms: i32,
        disconnect_timeout_ms: i32,
        time_control_ms: i32,
    ) -> Result<()> {
        let game_state = self.storage.create_game(
            Game::default(),
            join_timeout_ms,
            first_move_timeout_ms,
            disconnect_timeout_ms,
            time_control_ms,
            time_control_ms,
        )?;
        self.state.replace(Some(game_state));

        Ok(())
    }

    async fn handle_player_connected(&self, color: Color) -> Result<()> {
        let is_white_connected = self.is_player_connected(Color::White)?;
        let is_black_connected = self.is_player_connected(Color::Black)?;

        {
            let now = Date::now() as i64;
            let mut state = self.state_mut()?;
            let change = state.player_connected(PlayerConnected {
                color,
                now,
                is_white_connected,
                is_black_connected,
            });
            self.handle_state_change(&state, change, now)?;
        };

        self.schedule_next_alarm().await?;
        Ok(())
    }

    async fn handle_player_disconnected(&self, color: Color) -> Result<()> {
        if self.is_player_connected(color)? {
            return Ok(());
        }

        {
            let now = Date::now() as i64;
            let mut state = self.state_mut()?;
            let change = state.player_disconnected(color, now);
            self.handle_state_change(&state, change, now)?;
        }

        self.schedule_next_alarm().await?;
        Ok(())
    }

    fn handle_state_change(&self, state: &GameState, change: StateChange, now: i64) -> Result<()> {
        match change {
            StateChange::LifecycleChanged => {
                self.storage.save(state)?;

                let message = ServerMessage::Status(StatusMessage::new(state, now));
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
        let next_alarm = self.state()?.next_timeout_at();

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

    fn is_player_connected(&self, color: Color) -> Result<bool> {
        for socket in self.durable_state.get_websockets() {
            if socket.deserialize_attachment::<Color>()? == Some(color) {
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
