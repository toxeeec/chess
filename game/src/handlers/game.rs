use socketioxide::extract::{Data, SocketRef, State};

use crate::AppState;

pub(crate) async fn join_game(socket: SocketRef, Data(room): Data<String>, state: State<AppState>) {
    let _ = socket.leave_all();
    if state.games.read().await.contains(&room) {
        let _ = socket.join(room.clone());
        let _ = socket.to(room).emit("message", "user joined");
    } else {
        let _ = socket.disconnect();
    }
}
