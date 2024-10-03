use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    serve, Json, Router,
};
use serde::Deserialize;
use tokio::{net::TcpListener, sync::RwLock};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Default, Debug)]
struct AppState {
    games: Arc<RwLock<HashSet<String>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState::default());
    let app = Router::new()
        .route("/games", post(create_game))
        .route("/games/:id", get(join_game))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));
    let listener = TcpListener::bind("127.0.0.1:3001").await.unwrap();
    serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct CreateGamePayload {
    id: String,
}

async fn create_game(state: State<Arc<AppState>>, Json(payload): Json<CreateGamePayload>) {
    state.games.write().await.insert(payload.id);
}

async fn join_game(state: State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    if state.games.read().await.contains(&id) {
        ().into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
