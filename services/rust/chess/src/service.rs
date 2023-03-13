tonic::include_proto!("chess");

use chess::game::{moves::list::List, Game};
use chess_server::Chess;
use futures_core::Stream;
use rand::prelude::*;
use std::{pin::Pin, sync::Arc};
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex, RwLock,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Result, Status};

#[derive(Debug)]
pub struct Player {
    id: u32,
    sender: Sender<Result<JoinGameResponse, Status>>,
}

#[derive(Debug, Default)]
pub struct Players {
    player1: Option<Player>,
    player2: Option<Player>,
}
pub struct ChessService {
    game: Mutex<Game>,
    players: Arc<RwLock<Players>>,
}

impl ChessService {
    pub fn new() -> Self {
        Self {
            game: Mutex::new(Game::default()),
            players: Arc::new(RwLock::new(Players::default())),
        }
    }
}

struct Moves(Vec<String>);

impl From<&List> for Moves {
    fn from(value: &List) -> Self {
        let mut moves = Vec::with_capacity(value.0.len());
        for mov in &value.0 {
            moves.push(mov.to_string());
        }
        Self(moves)
    }
}

#[tonic::async_trait]
impl Chess for ChessService {
    async fn make_move(
        &self,
        req: Request<MakeMoveRequest>,
    ) -> Result<Response<MakeMoveResponse>, Status> {
        let req = req.into_inner();
        let mut game = self.game.lock().await;
        if let Err(err) = game.make_move(&req.r#move) {
            return Err(Status::new(Code::FailedPrecondition, err.to_string()));
        }
        let res = Ok(JoinGameResponse {
            fen: game.board.to_string(),
            result: game.result,
            moves: Moves::from(&game.list).0,
        });
        let mut players = self.players.write().await;
        if let Some(player1) = &players.player1 {
            let _ = player1.sender.send(res.clone()).await;
        };
        if let Some(player2) = &players.player2 {
            let _ = player2.sender.send(res.clone()).await;
        };
        if game.result.is_some() {
            players.player1 = None;
            players.player2 = None;
            *game = Game::default();
        }
        Ok(Response::new(MakeMoveResponse {}))
    }

    type JoinGameStream =
        Pin<Box<dyn Stream<Item = Result<JoinGameResponse, Status>> + Send + 'static>>;
    async fn join_game(
        &self,
        _: Request<JoinGameRequest>,
    ) -> Result<Response<Self::JoinGameStream>, Status> {
        let mut players = self.players.write().await;
        if players.player1.is_some() && players.player2.is_some() {
            return Err(Status::new(Code::FailedPrecondition, "Game is full"));
        };
        let id = rand::thread_rng().gen();
        let (stream_tx, stream_rx) = mpsc::channel(1);
        let (tx, mut rx) = mpsc::channel(1);
        let player = Some(Player { id, sender: tx });
        if players.player1.is_none() {
            players.player1 = player;
        } else {
            players.player2 = player;
        };

        let players = self.players.clone();
        tokio::spawn(async move {
            while let Some(res) = rx.recv().await {
                if stream_tx.send(res).await.is_err() {
                    let mut players = players.write().await;
                    if let Some(player) = &mut players.player1 {
                        if player.id == id {
                            players.player1 = None;
                        }
                    }
                    if let Some(player) = &mut players.player2 {
                        if player.id == id {
                            players.player2 = None;
                        }
                    }
                    break;
                }
            }
        });

        let output_stream = ReceiverStream::new(stream_rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::JoinGameStream
        ))
    }
}
