use crate::{
    game::{Game, MakeMoveError as GameMakeMoveError, Player},
    moves::{Move, MoveList},
};
use std::fmt;

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
    Ended,
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
    pub(crate) player: Player,
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
    pub(crate) const fn white_disconnected_at(&self) -> Option<i64> {
        match self.lifecycle {
            GameLifecycle::Active {
                white_disconnected_at,
                ..
            } => white_disconnected_at,
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended | GameLifecycle::Expired => None,
        }
    }

    pub(crate) const fn black_disconnected_at(&self) -> Option<i64> {
        match self.lifecycle {
            GameLifecycle::Active {
                black_disconnected_at,
                ..
            } => black_disconnected_at,
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended | GameLifecycle::Expired => None,
        }
    }

    pub(crate) const fn turn_started_at(&self) -> Option<i64> {
        match self.lifecycle {
            GameLifecycle::Active {
                turn_started_at, ..
            } => Some(turn_started_at),
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended | GameLifecycle::Expired => None,
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
            } => match event.player {
                Player::White => {
                    if white_disconnected_at.is_none() {
                        StateChange::Unchanged
                    } else {
                        *white_disconnected_at = None;
                        StateChange::Updated
                    }
                }
                Player::Black => {
                    if black_disconnected_at.is_none() {
                        StateChange::Unchanged
                    } else {
                        *black_disconnected_at = None;
                        StateChange::Updated
                    }
                }
            },
            GameLifecycle::Ended | GameLifecycle::Expired => StateChange::Unchanged,
        }
    }

    pub(super) fn player_disconnected(&mut self, player: Player, now: i64) -> StateChange {
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
        let disconnected_at = match player {
            Player::White => white_disconnected_at,
            Player::Black => black_disconnected_at,
        };

        if disconnected_at.is_some() {
            StateChange::Unchanged
        } else {
            *disconnected_at = Some(now);
            StateChange::Updated
        }
    }

    pub(super) fn process_due_event(&mut self, now: i64) -> StateChange {
        match &self.lifecycle {
            GameLifecycle::Waiting { created_at } => {
                debug_assert!(now >= *created_at);
                if created_at + self.timeouts.join_timeout_ms as i64 <= now {
                    self.lifecycle = GameLifecycle::Expired;
                    StateChange::LifecycleChanged
                } else {
                    StateChange::Unchanged
                }
            }
            GameLifecycle::Active {
                turn_started_at,
                white_disconnected_at,
                black_disconnected_at,
            } => {
                debug_assert!(now >= *turn_started_at);
                debug_assert!(
                    white_disconnected_at.is_none_or(|disconnected_at| now >= disconnected_at)
                );
                debug_assert!(
                    black_disconnected_at.is_none_or(|disconnected_at| now >= disconnected_at)
                );

                let first_move_expired = self.revision == 0
                    && turn_started_at + self.timeouts.first_move_timeout_ms as i64 <= now;
                let clock_expired =
                    self.revision > 0 && self.active_clock_expires_at(*turn_started_at) <= now;
                let white_disconnect_expired =
                    white_disconnected_at.is_some_and(|disconnected_at| {
                        disconnected_at + self.timeouts.disconnect_timeout_ms as i64 <= now
                    });
                let black_disconnect_expired =
                    black_disconnected_at.is_some_and(|disconnected_at| {
                        disconnected_at + self.timeouts.disconnect_timeout_ms as i64 <= now
                    });

                if first_move_expired
                    || clock_expired
                    || white_disconnect_expired
                    || black_disconnect_expired
                {
                    if clock_expired {
                        *self.remaining_ms_mut(self.game.turn) = 0;
                    }

                    self.lifecycle = if self.revision == 0 {
                        GameLifecycle::Expired
                    } else {
                        GameLifecycle::Ended
                    };
                    StateChange::LifecycleChanged
                } else {
                    StateChange::Unchanged
                }
            }
            GameLifecycle::Ended | GameLifecycle::Expired => StateChange::Unchanged,
        }
    }

    pub(super) fn next_event_at(&self) -> Option<i64> {
        match self.lifecycle {
            GameLifecycle::Waiting { created_at } => {
                Some(created_at + self.timeouts.join_timeout_ms as i64)
            }
            GameLifecycle::Active {
                turn_started_at,
                white_disconnected_at,
                black_disconnected_at,
            } => [
                (self.revision > 0).then_some(self.active_clock_expires_at(turn_started_at)),
                (self.revision == 0)
                    .then_some(turn_started_at + self.timeouts.first_move_timeout_ms as i64),
                white_disconnected_at.map(|disconnected_at| {
                    disconnected_at + self.timeouts.disconnect_timeout_ms as i64
                }),
                black_disconnected_at.map(|disconnected_at| {
                    disconnected_at + self.timeouts.disconnect_timeout_ms as i64
                }),
            ]
            .into_iter()
            .flatten()
            .min(),
            GameLifecycle::Ended | GameLifecycle::Expired => None,
        }
    }

    pub(super) fn make_move(
        &mut self,
        player: Player,
        mve: Move,
        now: i64,
    ) -> Result<(), MakeMoveError> {
        let Some(turn_started_at) = self.turn_started_at() else {
            return Err(MakeMoveError::GameNotActive);
        };
        debug_assert!(now >= turn_started_at);

        let moving_player = self.game.turn;
        self.game
            .make_move(player, mve)
            .map_err(MakeMoveError::from)?;

        if self.revision > 0 {
            let elapsed_ms = (now - turn_started_at) as i32;
            *self.remaining_ms_mut(moving_player) = self
                .remaining_ms(moving_player)
                .saturating_sub(elapsed_ms)
                .max(0);
        }
        if let GameLifecycle::Active {
            ref mut turn_started_at,
            ..
        } = self.lifecycle
        {
            *turn_started_at = now;
        }
        self.revision += 1;

        Ok(())
    }

    fn active_clock_expires_at(&self, turn_started_at: i64) -> i64 {
        turn_started_at + self.remaining_ms(self.game.turn) as i64
    }

    const fn remaining_ms(&self, player: Player) -> i32 {
        match player {
            Player::White => self.clock.white_remaining_ms,
            Player::Black => self.clock.black_remaining_ms,
        }
    }

    fn remaining_ms_mut(&mut self, player: Player) -> &mut i32 {
        match player {
            Player::White => &mut self.clock.white_remaining_ms,
            Player::Black => &mut self.clock.black_remaining_ms,
        }
    }

    pub(super) fn legal_moves(&self) -> &MoveList {
        match self.lifecycle {
            GameLifecycle::Active { .. } => &self.game.moves,
            GameLifecycle::Waiting { .. } | GameLifecycle::Ended | GameLifecycle::Expired => {
                MoveList::EMPTY
            }
        }
    }
}

impl fmt::Display for GameLifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Waiting { .. } => "waiting",
            Self::Active { .. } => "active",
            Self::Ended => "ended",
            Self::Expired => "expired",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

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
                Player::Black,
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
                player: Player::Black,
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

        assert_eq!(state.process_due_event(NOW), StateChange::LifecycleChanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Expired));
    }

    #[test]
    fn join_timeout_is_ignored_after_game_starts() {
        let mut state = GameState {
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        assert_eq!(
            state.process_due_event(NOW + JOIN_TIMEOUT_MS as i64),
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

        assert_eq!(state.process_due_event(NOW), StateChange::LifecycleChanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Expired));
    }

    #[test]
    fn first_move_timeout_is_ignored_after_first_move() {
        let mut state = GameState {
            revision: 1,
            lifecycle: active_lifecycle(NOW - FIRST_MOVE_TIMEOUT_MS as i64),
            ..test_state()
        };

        assert_eq!(state.process_due_event(NOW), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
    }

    #[test]
    fn first_move_starts_opponents_clock_without_decrementing_white_clock() {
        let mut state = GameState {
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        state
            .make_move(Player::White, Move::from_str("e2e3").unwrap(), NOW + 125)
            .unwrap();

        assert_eq!(state.clock.white_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.clock.black_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.turn_started_at(), Some(NOW + 125));
        assert_eq!(state.game.turn, Player::Black);
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

        assert_eq!(state.process_due_event(NOW + 50), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
    }

    #[test]
    fn move_after_first_move_decrements_moving_players_clock_and_switches_turn_start() {
        let mut state = after_white_move_state();

        state
            .make_move(Player::Black, Move::from_str("a7a6").unwrap(), NOW + 125)
            .unwrap();

        assert_eq!(state.clock.white_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.clock.black_remaining_ms, TIME_CONTROL_MS - 125);
        assert_eq!(state.turn_started_at(), Some(NOW + 125));
        assert_eq!(state.game.turn, Player::White);
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
            state.process_due_event(NOW + 50),
            StateChange::LifecycleChanged
        );
        assert!(matches!(state.lifecycle, GameLifecycle::Ended));
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
            state.process_due_event(NOW + 50),
            StateChange::LifecycleChanged
        );
        assert_eq!(state.clock.white_remaining_ms, TIME_CONTROL_MS);
        assert_eq!(state.clock.black_remaining_ms, 0);
    }

    #[test]
    fn next_event_includes_active_clock_timeout() {
        let state = GameState {
            revision: 1,
            clock: GameClock {
                white_remaining_ms: 50,
                black_remaining_ms: TIME_CONTROL_MS,
            },
            lifecycle: active_lifecycle(NOW),
            ..test_state()
        };

        assert_eq!(state.next_event_at(), Some(NOW + 50));
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

        assert_eq!(state.process_due_event(NOW), StateChange::LifecycleChanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Ended));
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

        assert_eq!(state.process_due_event(NOW), StateChange::LifecycleChanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Ended));
    }

    #[test]
    fn white_disconnect_timeout_expires_game_if_no_moves_made() {
        let mut state = GameState {
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: Some(NOW - DISCONNECT_TIMEOUT_MS as i64),
                black_disconnected_at: None,
            },
            ..test_state()
        };

        assert_eq!(state.process_due_event(NOW), StateChange::LifecycleChanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Expired));
    }

    #[test]
    fn black_disconnect_timeout_expires_game_if_no_moves_made() {
        let mut state = GameState {
            lifecycle: GameLifecycle::Active {
                turn_started_at: NOW,
                white_disconnected_at: None,
                black_disconnected_at: Some(NOW - DISCONNECT_TIMEOUT_MS as i64),
            },
            ..test_state()
        };

        assert_eq!(state.process_due_event(NOW), StateChange::LifecycleChanged);
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
            state.player_disconnected(Player::White, NOW),
            StateChange::Updated
        );
        assert_eq!(state.white_disconnected_at(), Some(NOW));
        assert_eq!(
            state.player_disconnected(Player::White, NOW + 1),
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
                player: Player::White,
                now: NOW,
                is_white_connected: true,
                is_black_connected: true,
            }),
            StateChange::Updated
        );
        assert_eq!(state.process_due_event(NOW), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Active { .. }));
        assert_eq!(state.white_disconnected_at(), None);
    }

    #[test]
    fn rejects_moves_unless_game_is_active() {
        for lifecycle in [
            GameLifecycle::Waiting { created_at: NOW },
            GameLifecycle::Ended,
            GameLifecycle::Expired,
        ] {
            let mut state = GameState {
                lifecycle,
                ..test_state()
            };

            assert!(matches!(
                state.make_move(Player::White, Move::from_str("e2e3").unwrap(), NOW),
                Err(MakeMoveError::GameNotActive)
            ));
            assert_eq!(state.revision, 0);
        }
    }

    #[test]
    fn expired_game_has_no_next_event() {
        let state = GameState {
            lifecycle: GameLifecycle::Expired,
            ..test_state()
        };

        assert_eq!(state.next_event_at(), None);
    }

    #[test]
    fn ended_game_ignores_due_events_and_has_no_next_event() {
        let mut state = GameState {
            lifecycle: GameLifecycle::Ended,
            ..test_state()
        };

        assert_eq!(state.process_due_event(NOW), StateChange::Unchanged);
        assert!(matches!(state.lifecycle, GameLifecycle::Ended));
        assert_eq!(state.next_event_at(), None);
    }
}
