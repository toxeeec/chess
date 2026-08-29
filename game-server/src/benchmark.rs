use crate::{game::Game, moves::UciMove};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub struct Perft {
    game: Game,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub struct GameReplay {
    games: Vec<Vec<UciMove>>,
}

impl GameReplay {
    fn from_json(dataset: &str) -> Result<Self, String> {
        let games = serde_json::from_str::<Vec<Vec<String>>>(dataset)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|moves| {
                moves
                    .into_iter()
                    .map(|mve| {
                        mve.parse()
                            .map_err(|error: anyhow::Error| error.to_string())
                    })
                    .collect()
            })
            .collect::<Result<_, _>>()?;

        let replay = Self { games };
        replay.validate()?;
        Ok(replay)
    }

    fn validate(&self) -> Result<(), String> {
        for (game_index, moves) in self.games.iter().enumerate() {
            let mut game = Game::default();
            for (move_index, &mve) in moves.iter().enumerate() {
                game.make_move(game.state.turn, mve).map_err(|_| {
                    format!(
                        "game {} move {} must be legal",
                        game_index + 1,
                        move_index + 1
                    )
                })?;
            }
        }

        Ok(())
    }

    fn execute(&self) -> usize {
        let mut count = 0;
        for moves in &self.games {
            let mut game = Game::default();
            for &mve in moves {
                // SAFETY: Construction validates every move from the same initial state.
                unsafe {
                    game.make_move(game.state.turn, std::hint::black_box(mve))
                        .unwrap_unchecked();
                }
                count += 1;
            }
        }

        count
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
impl GameReplay {
    #[cfg_attr(
        target_arch = "wasm32",
        wasm_bindgen::prelude::wasm_bindgen(constructor)
    )]
    pub fn new(dataset: &str) -> Result<Self, String> {
        Self::from_json(dataset)
    }

    pub fn run(&self) -> usize {
        self.execute()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
impl Perft {
    #[cfg_attr(
        target_arch = "wasm32",
        wasm_bindgen::prelude::wasm_bindgen(constructor)
    )]
    pub fn new(fen: &str) -> Result<Self, String> {
        Game::from_fen(fen)
            .map(|game| Self { game })
            .map_err(|error| error.to_string())
    }

    pub fn run(&self, depth: u32) -> u64 {
        self.game.perft(depth)
    }
}
