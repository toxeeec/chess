use super::{board::Board, moves, piece::ParsePieceError, state::State, Game};
use bitboard::square::{ParseSquareError, Square};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseGameError {
    #[error("fen must contain at least 4 fields")]
    Format,
    #[error("invalid state")]
    State(#[from] ParseStateError),
    #[error("invalid en passant target square")]
    EnPassant(#[from] ParseSquareError),
}

impl FromStr for Game {
    type Err = ParseGameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields: Vec<_> = s.split_whitespace().collect();
        fields.truncate(6);
        if fields.len() < 4 {
            return Err(ParseGameError::Format);
        }
        let board = Board::default();
        let state: State = fields[1..4].try_into()?;
        let mut moves = Vec::with_capacity(32);
        moves::generate(&mut moves, &board, state);
        Ok(Game {
            board,
            state,
            moves,
        })
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseBoardError {
    #[error("invalid piece")]
    Piece(#[from] ParsePieceError),
}

impl FromStr for Board {
    type Err = ParseBoardError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = Board::default();
        let mut sq = 56;
        for c in s.chars() {
            match c.try_into() {
                Ok(piece) => {
                    let is_white = c.is_uppercase();
                    board.get_mut(piece, is_white).set(sq);
                    sq += 1;
                }
                Err(err) => match c {
                    x @ '1'..='8' => sq += x.to_digit(10).unwrap(),
                    '/' => sq -= 16,
                    _ => return Err(ParseBoardError::Piece(err)),
                },
            };
        }
        Ok(board)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseStateError {
    #[error("side to move must be either \"w\" or \"b\" but was {0}")]
    SideToMove(String),
    #[error("castling ability cannot contain more than 4 characters")]
    CastlingFormat,
    #[error("castling ability must be either '-' or only contain 'KQkq' but contains {0}")]
    Castling(char),
    #[error("invalid en passant square format")]
    EnPassantFormat(#[from] ParseSquareError),
    #[error("en passant square rank must be either 3 or 6 but was {0}")]
    EnPassant(u32),
}

impl TryFrom<&[&str]> for State {
    type Error = ParseStateError;
    fn try_from(value: &[&str]) -> Result<Self, Self::Error> {
        let mut state = State {
            white: false,
            wk: false,
            wq: false,
            bk: false,
            bq: false,
            ep: None,
        };

        let side = value[0];
        match side {
            "w" => state.white = true,
            "b" => state.white = false,
            _ => return Err(ParseStateError::SideToMove(side.to_string())),
        }

        let castling = value[1];
        if castling.len() > 4 {
            return Err(ParseStateError::CastlingFormat);
        }
        if castling != "-" {
            for c in castling.chars() {
                match c {
                    'K' => state.wk = true,
                    'Q' => state.wq = true,
                    'k' => state.bk = true,
                    'q' => state.bq = true,
                    _ => return Err(ParseStateError::Castling(c)),
                }
            }
        }

        let ep = value[2];
        if ep != "-" {
            let ep = value[2].parse::<Square>()?;
            let rank = ep.rank_of();
            if rank != 3 && rank != 6 {
                return Err(ParseStateError::EnPassant(rank));
            }
            state.ep = Some(ep);
        }
        Ok(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" => Ok(Game::default()))]
    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq" => Err(ParseGameError::Format))]
    fn game_fromstr_tests(s: &str) -> Result<Game, ParseGameError> {
        s.parse()
    }

    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR" => Ok(Board::default()))]
    #[test_case("Anbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR" => Err(ParseBoardError::Piece(ParsePieceError::Unknown('A'))))]
    fn board_fromstr_tests(s: &str) -> Result<Board, ParseBoardError> {
        s.parse()
    }

    #[test_case(&["w", "KQkq", "-"] => Ok(State::default()))]
    #[test_case(&["a", "KQkq", "-"] => Err(ParseStateError::SideToMove("a".into())))]
    #[test_case(&["w", "KQkqK", "-"] => Err(ParseStateError::CastlingFormat))]
    #[test_case(&["w", "AQkq", "-"] => Err(ParseStateError::Castling('A')))]
    #[test_case(&["w", "KQkq", "aa9"] => Err(ParseStateError::EnPassantFormat(ParseSquareError::Format)))]
    #[test_case(&["w", "KQkq", "e4"] => Err(ParseStateError::EnPassant(4)))]
    fn state_tryfrom_tests(value: &[&str]) -> Result<State, ParseStateError> {
        value.try_into()
    }
}
