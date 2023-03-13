mod service;

use anyhow::Result;
use service::chess_server::ChessServer;
use service::ChessService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:10000".parse()?;
    let svc = ChessService::new();
    Server::builder()
        .add_service(ChessServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
