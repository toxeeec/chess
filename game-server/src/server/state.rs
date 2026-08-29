use crate::{
    game::{Game, GameResult, MakeMoveError as GameMakeMoveError},
    moves::{MoveList, UciMove},
    state::Color,
};
use serde::{Deserialize, Serialize};

pub(crate) struct GameTimeouts {
    pub(crate) join_timeout_ms: i32,
    pub(crate) first_move_timeout_ms: i32,
    pub(crate) disconnect_timeout_ms: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameClock {
    pub(crate) white_remaining_ms: i32,
    pub(crate) black_remaining_ms: i32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum WinReason {
    Checkmate,
    Timeout,
    Disconnect,
}

impl WinReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Checkmate => "checkmate",
            Self::Timeout => "timeout",
            Self::Disconnect => "disconnect",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum DrawReason {
    Stalemate,
}

impl DrawReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Stalemate => "stalemate",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum GameOutcome {
    Won { winner: Color, reason: WinReason },
    Drawn { reason: DrawReason },
}

impl GameOutcome {
    pub(crate) fn winner(self) -> Option<Color> {
        match self {
            Self::Won { winner, .. } => Some(winner),
            Self::Drawn { .. } => None,
        }
    }

    pub(crate) fn reason(self) -> &'static str {
        match self {
            Self::Won { reason, .. } => reason.as_str(),
            Self::Drawn { reason } => reason.as_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GameLifecycle {
    Waiting {
        created_at: i64,
    },
    Active {
        turn_started_at: i64,
        white_disconnected_at: Option<i64>,
        black_disconnected_at: Option<i64>,
    },
    Ended(GameOutcome),
    Expired,
}

pub(crate) struct GameState {
    pub(crate) game: Game,
    pub(crate) lifecycle: GameLifecycle,
    pub(crate) timeouts: GameTimeouts,
    pub(crate) clock: GameClock,
    pub(crate) revision: u32,
}

pub(crate) struct PlayerConnected {
    pub(crate) color: Color,
    pub(crate) now: i64,
    pub(crate) is_white_connected: bool,
    pub(crate) is_black_connected: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StateChange {
    Unchanged,
    Updated,
    LifecycleChanged,
}

#[derive(Clone, Copy)]
enum Timeout {
    Join,
    FirstMove,
    Clock { color: Color },
    Disconnect { color: Color },
}

#[derive(Clone, Copy)]
struct ScheduledTimeout {
    at: i64,
    timeout: Timeout,
}

#[derive(Debug)]
pub(super) enum MakeMoveError {
    GameNotActive,
    IllegalMove,
    NotYourTurn,
}

impl From<GameMakeMoveError> for MakeMoveError {
    fn from(error: GameMakeMoveError) -> Self {
        match error {
            GameMakeMoveError::IllegalMove => Self::IllegalMove,
            GameMakeMoveError::NotYourTurn => Self::NotYourTurn,
        }
    }
}

impl GameState {
    pub(crate) fn white_disconnected_at(&self) -> Option<i64> {
        match self.lifecycle {
            GameLifecycle::Active {
                white_disconnected_at,
                ..
            } => white_disconnected_at,
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended(_) | GameLifecycle::Expired => {
                None
            }
        }
    }

    pub(crate) fn black_disconnected_at(&self) -> Option<i64> {
        match self.lifecycle {
            GameLifecycle::Active {
                black_disconnected_at,
                ..
            } => black_disconnected_at,
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended(_) | GameLifecycle::Expired => {
                None
            }
        }
    }

    pub(crate) fn turn_started_at(&self) -> Option<i64> {
        match self.lifecycle {
            GameLifecycle::Active {
                turn_started_at, ..
            } => Some(turn_started_at),
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended(_) | GameLifecycle::Expired => {
                None
            }
        }
    }

    pub(super) fn player_connected(&mut self, event: PlayerConnected) -> StateChange {
        match &mut self.lifecycle {
            GameLifecycle::Waiting { .. } => {
                if event.is_white_connected && event.is_black_connected {
                    self.lifecycle = GameLifecycle::Active {
                        turn_started_at: event.now,
                        white_disconnected_at: None,
                        black_disconnected_at: None,
                    };
                    StateChange::LifecycleChanged
                } else {
                    StateChange::Unchanged
                }
            }
            GameLifecycle::Active {
                white_disconnected_at,
                black_disconnected_at,
                ..
            } => match event.color {
                Color::White => {
                    if white_disconnected_at.is_none() {
                        StateChange::Unchanged
                    } else {
                        *white_disconnected_at = None;
                        StateChange::Updated
                    }
                }
                Color::Black => {
                    if black_disconnected_at.is_none() {
                        StateChange::Unchanged
                    } else {
                        *black_disconnected_at = None;
                        StateChange::Updated
                    }
                }
            },
            GameLifecycle::Ended(_) | GameLifecycle::Expired => StateChange::Unchanged,
        }
    }

    pub(super) fn player_disconnected(&mut self, color: Color, now: i64) -> StateChange {
        let GameLifecycle::Active {
            turn_started_at,
            white_disconnected_at,
            black_disconnected_at,
            ..
        } = &mut self.lifecycle
        else {
            return StateChange::Unchanged;
        };

        debug_assert!(now >= *turn_started_at);
        let disconnected_at = match color {
            Color::White => white_disconnected_at,
            Color::Black => black_disconnected_at,
        };

        if disconnected_at.is_some() {
            StateChange::Unchanged
        } else {
            *disconnected_at = Some(now);
            StateChange::Updated
        }
    }

    pub(super) fn process_due_timeout(&mut self, now: i64) -> StateChange {
        let Some(scheduled) = self.next_timeout() else {
            return StateChange::Unchanged;
        };
        if now < scheduled.at {
            return StateChange::Unchanged;
        }

        self.lifecycle = match scheduled.timeout {
            Timeout::Join | Timeout::FirstMove => GameLifecycle::Expired,
            Timeout::Clock { color } => {
                *self.remaining_ms_mut(color) = 0;
                GameLifecycle::Ended(GameOutcome::Won {
                    winner: color.opponent(),
                    reason: WinReason::Timeout,
                })
            }
            Timeout::Disconnect { color } => {
                if self.revision == 0 {
                    GameLifecycle::Expired
                } else {
                    GameLifecycle::Ended(GameOutcome::Won {
                        winner: color.opponent(),
                        reason: WinReason::Disconnect,
                    })
                }
            }
        };
        StateChange::LifecycleChanged
    }

    pub(super) fn next_timeout_at(&self) -> Option<i64> {
        self.next_timeout().map(|scheduled| scheduled.at)
    }

    fn next_timeout(&self) -> Option<ScheduledTimeout> {
        match self.lifecycle {
            GameLifecycle::Waiting { created_at } => Some(ScheduledTimeout {
                at: created_at + self.timeouts.join_timeout_ms as i64,
                timeout: Timeout::Join,
            }),
            GameLifecycle::Active {
                turn_started_at,
                white_disconnected_at,
                black_disconnected_at,
            } => [
                (self.revision == 0).then_some(ScheduledTimeout {
                    at: turn_started_at + self.timeouts.first_move_timeout_ms as i64,
                    timeout: Timeout::FirstMove,
                }),
                (self.revision > 0).then_some(ScheduledTimeout {
                    at: self.active_clock_expires_at(turn_started_at),
                    timeout: Timeout::Clock {
                        color: self.game.state.turn,
                    },
                }),
                white_disconnected_at.map(|disconnected_at| ScheduledTimeout {
                    at: disconnected_at + self.timeouts.disconnect_timeout_ms as i64,
                    timeout: Timeout::Disconnect {
                        color: Color::White,
                    },
                }),
                black_disconnected_at.map(|disconnected_at| ScheduledTimeout {
                    at: disconnected_at + self.timeouts.disconnect_timeout_ms as i64,
                    timeout: Timeout::Disconnect {
                        color: Color::Black,
                    },
                }),
            ]
            .into_iter()
            .flatten()
            .min_by_key(|scheduled| scheduled.at),
            GameLifecycle::Ended(_) | GameLifecycle::Expired => None,
        }
    }

    pub(super) fn make_move(
        &mut self,
        color: Color,
        mve: UciMove,
        now: i64,
    ) -> Result<(), MakeMoveError> {
        let Some(turn_started_at) = self.turn_started_at() else {
            return Err(MakeMoveError::GameNotActive);
        };
        debug_assert!(now >= turn_started_at);

        let moving_color = self.game.state.turn;
        let result = self
            .game
            .make_move(color, mve)
            .map_err(MakeMoveError::from)?;

        if self.revision > 0 {
            let elapsed_ms = (now - turn_started_at) as i32;
            *self.remaining_ms_mut(moving_color) = self
                .remaining_ms(moving_color)
                .saturating_sub(elapsed_ms)
                .max(0);
        }
        if let GameLifecycle::Active {
            turn_started_at, ..
        } = &mut self.lifecycle
        {
            *turn_started_at = now;
        }
        self.revision += 1;

        if let Some(result) = result {
            self.lifecycle = match result {
                GameResult::Win { winner } => GameLifecycle::Ended(GameOutcome::Won {
                    winner,
                    reason: WinReason::Checkmate,
                }),
                GameResult::Draw => GameLifecycle::Ended(GameOutcome::Drawn {
                    reason: DrawReason::Stalemate,
                }),
            };
        }

        Ok(())
    }

    fn active_clock_expires_at(&self, turn_started_at: i64) -> i64 {
        turn_started_at + self.remaining_ms(self.game.state.turn) as i64
    }

    fn remaining_ms(&self, color: Color) -> i32 {
        match color {
            Color::White => self.clock.white_remaining_ms,
            Color::Black => self.clock.black_remaining_ms,
        }
    }

    fn remaining_ms_mut(&mut self, color: Color) -> &mut i32 {
        match color {
            Color::White => &mut self.clock.white_remaining_ms,
            Color::Black => &mut self.clock.black_remaining_ms,
        }
    }

    pub(super) fn legal_moves(&self) -> &MoveList {
        match self.lifecycle {
            GameLifecycle::Active { .. } => &self.game.moves,
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended(_) | GameLifecycle::Expired => {
                MoveList::EMPTY
            }
        }
    }

    pub(super) fn winner(&self) -> Option<Color> {
        match self.lifecycle {
            GameLifecycle::Ended(outcome) => outcome.winner(),
            GameLifecycle::Waiting { .. }
            | GameLifecycle::Active { .. }
            | GameLifecycle::Expired => None,
        }
    }

    pub(super) fn end_reason(&self) -> Option<&'static str> {
        match self.lifecycle {
            GameLifecycle::Ended(outcome) => Some(outcome.reason()),
            GameLifecycle::Waiting { .. }
            | GameLifecycle::Active { .. }
            | GameLifecycle::Expired => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{castling::CastlingRights, state::State};

    const NOW: i64 = 1_000;
    const JOIN_TIMEOUT_MS: i32 = 100;
    const FIRST_MOVE_TIMEOUT_MS: i32 = 200;
    const DISCONNECT_TIMEOUT_MS: i32 = 300;
    const TIME_CONTROL_MS: i32 = 1_000;
    const TEST_TIMEOUTS: GameTimeouts = GameTimeouts {
        join_timeout_ms: JOIN_TIMEOUT_MS,
        first_move_timeout_ms: FIRST_MOVE_TIMEOUT_MS,
        disconnect_timeout_ms: DISCONNECT_TIMEOUT_MS,
    };
    const TEST_CLOCKS: GameClock = GameClock {
        white_remaining_ms: TIME_CONTROL_MS,
        black_remaining_ms: TIME_CONTROL_MS,
    };

    fn test_state() -> GameState {
        GameState {
            game: Game::default(),
            revision: 0,
            timeouts: TEST_TIMEOUTS,
            clock: TEST_CLOCKS,
            lifecycle: GameLifecycle::Waiting { created_at: NOW },
        }
    }

    fn active_lifecycle(turn_started_at: i64) -> GameLifecycle {
        GameLifecycle::Active {
            turn_started_at,
            white_disconnected_at: None,
            black_disconnected_at: None,
        }
    }

    fn after_white_move_state() -> GameState {
        GameState {
            game: Game::new(
                crate::board::Board::from_fen("rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR")
                    .unwrap(),
                State::new(Color::Black, CastlingRights::NONE),
            ),
            revision: 1,
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        }
    }

    #[test]
    fn second_player_connected_starts_waiting_game() {
        let mut state = test_state();

        assert_eq!(
            state.player_connected(PlayerConnected {
                color: Color::Black,
                now: NOW,
                is_white_connected: true,
                is_black_connected: true,
            }),
            StateChange::LifecycleChanged
        );
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
    }

    #[test]
    fn join_timeout_expires_waiting_game() {
        let mut state = GameState {
            lifecycle: GameLifecycle::Waiting {
                created_at: NOW - JOIN_TIMEOUT_MS as i64,
            },
            ..test_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW),
            StateChange::LifecycleChanged
        );
        assert!(matches!(state.lifecycle, GameLifecycle::Expired));
    }

    #[test]
    fn join_timeout_is_ignored_after_game_starts() {
        let mut state = GameState {
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW + JOIN_TIMEOUT_MS as i64),
            StateChange::Unchanged
        );
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
    }

    #[test]
    fn first_move_timeout_expires_active_game_at_revision_zero() {
        let mut state = GameState {
            lifecycle: active_lifecycle(NOW - FIRST_MOVE_TIMEOUT_MS as i64),
            ..test_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW),
            StateChange::LifecycleChanged
        );
        assert!(matches!(state.lifecycle, GameLifecycle::Expired));
    }

    #[test]
    fn first_move_timeout_is_ignored_after_first_move() {
        let mut state = GameState {
            revision: 1,
            lifecycle: active_lifecycle(NOW - FIRST_MOVE_TIMEOUT_MS as i64),
            ..test_state()
        };

        assert_eq!(state.process_due_timeout(NOW), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
    }

    #[test]
    fn first_move_starts_opponents_clock_without_decrementing_white_clock() {
        let mut state = GameState {
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        state
            .make_move(Color::White, "e2e3".parse().unwrap(), NOW + 125)
            .unwrap();

        assert_eq!(state.clock.white_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.clock.black_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.turn_started_at(), Some(NOW + 125));
        assert_eq!(state.game.state.turn, Color::Black);
    }

    #[test]
    fn active_clock_timeout_is_ignored_before_first_move() {
        let mut state = GameState {
            clock: GameClock {
                white_remaining_ms: 50,
                black_remaining_ms: TIME_CONTROL_MS,
            },
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        assert_eq!(state.process_due_timeout(NOW + 50), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
    }

    #[test]
    fn move_after_first_move_decrements_moving_players_clock_and_switches_turn_start() {
        let mut state = after_white_move_state();

        state
            .make_move(Color::Black, "a7a6".parse().unwrap(), NOW + 125)
            .unwrap();

        assert_eq!(state.clock.white_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.clock.black_remaining_ms, TIME_CONTROL_MS - 125);
        assert_eq!(state.turn_started_at(), Some(NOW + 125));
        assert_eq!(state.game.state.turn, Color::White);
    }

    #[test]
    fn active_clock_timeout_ends_game_after_a_move() {
        let mut state = GameState {
            clock: GameClock {
                white_remaining_ms: TIME_CONTROL_MS,
                black_remaining_ms: 50,
            },
            ..after_white_move_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW + 50),
            StateChange::LifecycleChanged
        );
        assert!(matches!(
            state.lifecycle,
            GameLifecycle::Ended(GameOutcome::Won {
                winner: Color::White,
                reason: WinReason::Timeout,
            })
        ));
    }

    #[test]
    fn checkmate_ends_game_and_rejects_more_moves() {
        let mut state = GameState {
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        for (color, mve) in [
            (Color::White, "f2f3"),
            (Color::Black, "e7e5"),
            (Color::White, "g2g4"),
            (Color::Black, "d8h4"),
        ] {
            state.make_move(color, mve.parse().unwrap(), NOW).unwrap();
        }

        assert!(matches!(
            state.lifecycle,
            GameLifecycle::Ended(GameOutcome::Won {
                winner: Color::Black,
                reason: WinReason::Checkmate,
            })
        ));
        assert_eq!(state.winner(), Some(Color::Black));
        assert!(state.legal_moves().is_empty());
        assert!(matches!(
            state.make_move(Color::White, "a2a3".parse().unwrap(), NOW),
            Err(MakeMoveError::GameNotActive)
        ));
    }

    #[test]
    fn stalemate_ends_game_without_a_winner() {
        let mut state = GameState {
            game: Game::from_fen("7k/5K2/8/6Q1/8/8/8/8 w - - 0 1").unwrap(),
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        state
            .make_move(Color::White, "g5g6".parse().unwrap(), NOW)
            .unwrap();

        assert!(matches!(
            state.lifecycle,
            GameLifecycle::Ended(GameOutcome::Drawn {
                reason: DrawReason::Stalemate,
            })
        ));
        assert_eq!(state.winner(), None);
        assert!(state.legal_moves().is_empty());
        assert!(matches!(
            state.make_move(Color::Black, "h8g8".parse().unwrap(), NOW),
            Err(MakeMoveError::GameNotActive)
        ));
    }

    #[test]
    fn active_clock_timeout_sets_losing_players_clock_to_zero() {
        let mut state = GameState {
            clock: GameClock {
                white_remaining_ms: TIME_CONTROL_MS,
                black_remaining_ms: 50,
            },
            ..after_white_move_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW + 50),
            StateChange::LifecycleChanged
        );
        assert_eq!(state.clock.white_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.clock.black_remaining_ms, 0);
    }

    #[test]
    fn next_timeout_includes_active_clock() {
        let state = GameState {
            revision: 1,
            clock: GameClock {
                white_remaining_ms: 50,
                black_remaining_ms: TIME_CONTROL_MS,
            },
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        assert_eq!(state.next_timeout_at(), Some(NOW + 50));
    }

    #[test]
    fn disconnect_timeout_ends_game_if_still_disconnected() {
        let mut state = GameState {
            revision: 1,
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: Some(NOW - DISCONNECT_TIMEOUT_MS as i64),
                black_disconnected_at: None,
            },
            ..test_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW),
            StateChange::LifecycleChanged
        );
        assert!(matches!(
            state.lifecycle,
            GameLifecycle::Ended(GameOutcome::Won {
                winner: Color::Black,
                reason: WinReason::Disconnect,
            })
        ));
    }

    #[test]
    fn black_disconnect_timeout_ends_game_after_first_move() {
        let mut state = GameState {
            revision: 1,
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: None,
                black_disconnected_at: Some(NOW - DISCONNECT_TIMEOUT_MS as i64),
            },
            ..test_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW),
            StateChange::LifecycleChanged
        );
        assert!(matches!(
            state.lifecycle,
            GameLifecycle::Ended(GameOutcome::Won {
                winner: Color::White,
                reason: WinReason::Disconnect,
            })
        ));
    }

    #[test]
    fn white_disconnect_timeout_expires_game_if_no_moves_were_made() {
        let mut state = GameState {
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: Some(NOW - DISCONNECT_TIMEOUT_MS as i64),
                black_disconnected_at: None,
            },
            ..test_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW),
            StateChange::LifecycleChanged
        );
        assert!(matches!(state.lifecycle, GameLifecycle::Expired));
    }

    #[test]
    fn black_disconnect_timeout_expires_game_if_no_moves_were_made() {
        let mut state = GameState {
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: None,
                black_disconnected_at: Some(NOW - DISCONNECT_TIMEOUT_MS as i64),
            },
            ..test_state()
        };

        assert_eq!(
            state.process_due_timeout(NOW),
            StateChange::LifecycleChanged
        );
        assert!(matches!(state.lifecycle, GameLifecycle::Expired));
    }

    #[test]
    fn player_disconnected_updates_active_game_once() {
        let mut state = GameState {
            revision: 1,
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        assert_eq!(
            state.player_disconnected(Color::White, NOW),
            StateChange::Updated
        );
        assert_eq!(state.white_disconnected_at(), Some(NOW));
        assert_eq!(
            state.player_disconnected(Color::White, NOW + 1),
            StateChange::Unchanged
        );
        assert_eq!(state.white_disconnected_at(), Some(NOW));
    }

    #[test]
    fn disconnect_timeout_is_ignored_after_reconnect() {
        let mut state = GameState {
            revision: 1,
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: Some(NOW - DISCONNECT_TIMEOUT_MS as i64),
                black_disconnected_at: None,
            },
            ..test_state()
        };

        assert_eq!(
            state.player_connected(PlayerConnected {
                color: Color::White,
                now: NOW,
                is_white_connected: true,
                is_black_connected: true,
            }),
            StateChange::Updated
        );
        assert_eq!(state.process_due_timeout(NOW), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
        assert_eq!(state.white_disconnected_at(), None);
    }

    #[test]
    fn rejects_moves_unless_game_is_active() {
        for lifecycle in [
            GameLifecycle::Waiting { created_at: NOW },
            GameLifecycle::Ended(GameOutcome::Won {
                winner: Color::White,
                reason: WinReason::Checkmate,
            }),
            GameLifecycle::Expired,
        ] {
            let mut state = GameState {
                lifecycle,
                ..test_state()
            };

            assert!(matches!(
                state.make_move(Color::White, "e2e3".parse().unwrap(), NOW),
                Err(MakeMoveError::GameNotActive)
            ));
            assert_eq!(state.revision, 0);
        }
    }

    #[test]
    fn expired_game_has_no_next_timeout() {
        let state = GameState {
            lifecycle: GameLifecycle::Expired,
            ..test_state()
        };

        assert_eq!(state.next_timeout_at(), None);
    }

    #[test]
    fn ended_game_ignores_due_timeouts_and_has_no_next_timeout() {
        let mut state = GameState {
            lifecycle: GameLifecycle::Ended(GameOutcome::Won {
                winner: Color::White,
                reason: WinReason::Checkmate,
            }),
            ..test_state()
        };

        assert_eq!(state.process_due_timeout(NOW), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Ended(_)));
        assert_eq!(state.next_timeout_at(), None);
    }
}
