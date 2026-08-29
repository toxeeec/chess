use crate::{
    board::Board,
    game::{Game, generate_legal_moves, make_move, unmake_move},
    moves::MoveList,
    state::State,
};

impl Game {
    pub(super) fn perft(&self, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }
        if depth == 1 {
            return self.moves.len() as u64;
        }

        let mut board = self.board;
        let mut state = self.state;
        let mut move_lists = std::iter::repeat_with(MoveList::default)
            .take(depth as usize - 1)
            .collect::<Vec<_>>();

        self.moves
            .iter()
            .map(|mve| {
                // SAFETY: `mve` was generated for the copied board and state.
                let undo = unsafe { make_move(&mut board, &mut state, mve) };
                let nodes = perft(&mut board, &mut state, depth - 1, move_lists.as_mut_slice());
                unmake_move(&mut board, &mut state, mve, undo);
                nodes
            })
            .sum()
    }
}

fn perft(board: &mut Board, state: &mut State, depth: u32, move_lists: &mut [MoveList]) -> u64 {
    let (moves, child_move_lists) = move_lists
        .split_first_mut()
        .expect("perft must allocate one move list per searched ply");
    generate_legal_moves(board, *state, moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    moves
        .iter()
        .map(|mve| {
            // SAFETY: `moves` was generated for the current board and state above.
            let undo = unsafe { make_move(board, state, mve) };
            let nodes = perft(board, state, depth - 1, child_move_lists);
            unmake_move(board, state, mve, undo);
            nodes
        })
        .sum()
}
