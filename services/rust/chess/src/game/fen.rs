use super::{board::Board, counter::Counter, piece::ParsePieceError, state::State, Game};
use bitboard::square::{ParseSquareError, Square};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseGameError {
    #[error("fen must contain at least 4 fields")]
    Format,
    #[error("invalid piece placement field")]
    PiecePlacement(#[from] ParseBoardError),
    #[error("invalid state")]
    State(#[from] ParseStateError),
    #[error("invalid en passant target square")]
    EnPassant(#[from] ParseSquareError),
    #[error("invalid counters")]
    Counter(#[from] ParseCounterError),
}

impl FromStr for Game {
    type Err = ParseGameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields: Vec<_> = s.split_whitespace().collect();
        fields.truncate(6);
        if fields.len() < 4 {
            return Err(ParseGameError::Format);
        }
        let board = fields[0].parse()?;
        let state = fields[1..4].try_into()?;
        let counter = if fields.len() == 6 {
            fields[4..6].try_into()?
        } else {
            Counter::default()
        };
        Ok(Game::new(board, state, counter))
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
        let mut board = Board::empty();
        let mut sq = 56;
        for c in s.chars() {
            match c.try_into() {
                Ok(piece) => {
                    let is_white = c.is_uppercase();
                    board.get_mut(piece, is_white).set(sq.into());
                    sq += 1;
                }
                Err(err) => match c {
                    x @ '1'..='8' => sq += x.to_digit(10).unwrap(),
                    '/' => sq -= 16,
                    _ => return Err(ParseBoardError::Piece(err)),
                },
            };
        }
        board.set_white();
        board.set_black();
        board.set_occ();
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
            let rank = ep.rank() + 1;
            if rank != 3 && rank != 6 {
                return Err(ParseStateError::EnPassant(rank));
            }
            state.ep = Some(ep);
        }
        Ok(state)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseCounterError {
    #[error("halfmove clock must be a number but was {0}")]
    HalfMoveFormat(String),
    #[error("fullmove counter must be a number but was {0}")]
    FullMoveFormat(String),
    #[error("halfmove clock must be a positive number but was {0}")]
    HalfMove(i32),
    #[error("fullmove counter must be greater than 0 but was {0}")]
    FullMove(i32),
}

impl TryFrom<&[&str]> for Counter {
    type Error = ParseCounterError;
    fn try_from(value: &[&str]) -> Result<Self, Self::Error> {
        let half = value[0]
            .parse::<i32>()
            .map_err(|_| ParseCounterError::HalfMoveFormat(value[0].to_string()))
            .and_then(|x| x.try_into().map_err(|_| ParseCounterError::HalfMove(x)))?;
        let full = match value[1].parse::<i32>() {
            Ok(x) => {
                if x > 0 {
                    x.try_into().unwrap()
                } else {
                    return Err(ParseCounterError::FullMove(x));
                }
            }
            Err(_) => return Err(ParseCounterError::FullMoveFormat(value[1].to_string())),
        };
        Ok(Self { half, full })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" => Ok(Game::default().board))]
    #[test_case("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq" => Err(ParseGameError::Format))]
    fn game_fromstr_tests(s: &str) -> Result<Board, ParseGameError> {
        match s.parse::<Game>() {
            Ok(game) => Ok(game.board),
            Err(e) => Err(e),
        }
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

    #[test_case(&["0", "1"] => Ok(Counter::default()))]
    #[test_case(&["a", "1"] => Err(ParseCounterError::HalfMoveFormat("a".into())))]
    #[test_case(&["0", "a"] => Err(ParseCounterError::FullMoveFormat("a".into())))]
    #[test_case(&["-1", "1"] => Err(ParseCounterError::HalfMove(-1)))]
    #[test_case(&["0", "-1"] => Err(ParseCounterError::FullMove(-1)))]
    #[test_case(&["0", "0"] => Err(ParseCounterError::FullMove(0)))]
    fn counter_tryfrom_tests(value: &[&str]) -> Result<Counter, ParseCounterError> {
        value.try_into()
    }
}
