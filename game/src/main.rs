use std::{collections::HashSet, sync::Arc};

use axum::{routing::post, serve, Router};
use handlers::{admin::create_game, game::join_game};
use socketioxide::{extract::SocketRef, SocketIo};
use tokio::{net::TcpListener, sync::RwLock};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

mod handlers;

#[derive(Default, Clone, Debug)]
struct AppState {
    games: Arc<RwLock<HashSet<String>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(LevelFilter::DEBUG))
        .init();

    let state = AppState::default();

    let (layer, io) = SocketIo::builder().with_state(state.clone()).build_layer();

    io.ns("/", |s: SocketRef| {
        s.on("join", join_game);
    });

    let app = Router::new()
        .route("/games", post(create_game))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(layer),
        );

    let listener = TcpListener::bind("127.0.0.1:3001").await.unwrap();
    serve(listener, app).await.unwrap();
}
