use axum::{extract::State, Json};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateGamePayload {
    id: String,
}

pub(crate) async fn create_game(state: State<AppState>, Json(payload): Json<CreateGamePayload>) {
    state.games.write().await.insert(payload.id);
}
