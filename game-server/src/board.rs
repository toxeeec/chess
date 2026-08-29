use std::{
    fmt,
    ops::{Index, IndexMut},
};

use anyhow::{Result, bail};

use crate::{
    bitboard::Bitboard,
    moves::{Move, MoveKind, MoveList, PromotionPiece},
    square,
    square::Square,
    squares,
    state::Color,
};

const CASTLING_ROOK_MOVES: [[(Square, Square); 2]; 2] = [
    [(square!(a1), square!(d1)), (square!(h1), square!(f1))],
    [(square!(a8), square!(d8)), (square!(h8), square!(f8))],
];

#[derive(Clone, Copy)]
pub(super) struct Board {
    pieces: [[Bitboard; 6]; 2],
    mailbox: [MailboxEntry; 64],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum Piece {
    Knight = 0,
    Bishop = 1,
    Rook = 2,
    Queen = 3,
    Pawn = 4,
    King = 5,
}

impl Piece {
    const ALL: [Self; 6] = [
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::Pawn,
        Self::King,
    ];
    const FEN: [[char; 6]; 2] = [
        ['N', 'B', 'R', 'Q', 'P', 'K'],
        ['n', 'b', 'r', 'q', 'p', 'k'],
    ];

    /// # Safety
    ///
    /// `code` must be a valid `Piece` discriminant in `0..=5`.
    unsafe fn from_code_unchecked(code: u8) -> Self {
        debug_assert!(code < Self::ALL.len() as u8);

        // SAFETY: `Piece` has contiguous `repr(u8)` discriminants from 0 through 5.
        unsafe { std::mem::transmute(code) }
    }

    fn from_fen(value: char) -> Option<(Color, Self)> {
        let index = Self::FEN
            .iter()
            .flatten()
            .position(|piece| *piece == value)?;
        let piece_count = Self::ALL.len();

        Some((
            Color::ALL[index / piece_count],
            Self::ALL[index % piece_count],
        ))
    }

    fn fen(self, color: Color) -> char {
        Self::FEN[color][self]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
struct MailboxEntry(u8);

impl MailboxEntry {
    const PIECE_MASK: u8 = 0b111;
    const EMPTY: Self = Self(Self::PIECE_MASK);

    const fn new(color: Color, piece: Piece) -> Self {
        Self(piece as u8 | ((color as u8) << 3))
    }

    /// # Safety
    ///
    /// This entry must contain a piece.
    unsafe fn piece_unchecked(self) -> Piece {
        let code = self.0 & Self::PIECE_MASK;
        debug_assert!(code < Piece::ALL.len() as u8);

        // SAFETY: By this function's contract, `code` is a valid `Piece` discriminant.
        unsafe { Piece::from_code_unchecked(code) }
    }

    fn color(self) -> Option<Color> {
        if self == Self::EMPTY {
            return None;
        }

        Some(Color::ALL[usize::from((self.0 >> 3) & 1)])
    }

    #[cfg(any(test, feature = "benchmark"))]
    fn piece(self) -> Option<Piece> {
        if self == Self::EMPTY {
            return None;
        }

        // SAFETY: Every non-empty `MailboxEntry` contains a valid piece.
        Some(unsafe { self.piece_unchecked() })
    }

    fn colored_piece(self) -> Option<(Color, Piece)> {
        let color = self.color()?;
        let code = self.0 & Self::PIECE_MASK;
        debug_assert!(code < Piece::ALL.len() as u8);

        // SAFETY: By `MailboxEntry`'s invariant, every non-empty entry contains
        // a valid `Piece` discriminant.
        let piece = unsafe { Piece::from_code_unchecked(code) };

        Some((color, piece))
    }
}

#[derive(Clone, Copy)]
pub(super) struct BoardUndo {
    #[cfg_attr(not(any(test, feature = "benchmark")), allow(dead_code))]
    moved: MailboxEntry,
    #[cfg_attr(not(any(test, feature = "benchmark")), allow(dead_code))]
    captured: MailboxEntry,
}

const DEFAULT_MAILBOX: [MailboxEntry; 64] = {
    let mut mailbox = [MailboxEntry::EMPTY; 64];
    mailbox[0] = MailboxEntry::new(Color::White, Piece::Rook);
    mailbox[1] = MailboxEntry::new(Color::White, Piece::Knight);
    mailbox[2] = MailboxEntry::new(Color::White, Piece::Bishop);
    mailbox[3] = MailboxEntry::new(Color::White, Piece::Queen);
    mailbox[4] = MailboxEntry::new(Color::White, Piece::King);
    mailbox[5] = MailboxEntry::new(Color::White, Piece::Bishop);
    mailbox[6] = MailboxEntry::new(Color::White, Piece::Knight);
    mailbox[7] = MailboxEntry::new(Color::White, Piece::Rook);
    mailbox[56] = MailboxEntry::new(Color::Black, Piece::Rook);
    mailbox[57] = MailboxEntry::new(Color::Black, Piece::Knight);
    mailbox[58] = MailboxEntry::new(Color::Black, Piece::Bishop);
    mailbox[59] = MailboxEntry::new(Color::Black, Piece::Queen);
    mailbox[60] = MailboxEntry::new(Color::Black, Piece::King);
    mailbox[61] = MailboxEntry::new(Color::Black, Piece::Bishop);
    mailbox[62] = MailboxEntry::new(Color::Black, Piece::Knight);
    mailbox[63] = MailboxEntry::new(Color::Black, Piece::Rook);

    let mut file = 0;
    while file < 8 {
        mailbox[8 + file] = MailboxEntry::new(Color::White, Piece::Pawn);
        mailbox[48 + file] = MailboxEntry::new(Color::Black, Piece::Pawn);
        file += 1;
    }

    mailbox
};

impl Default for Board {
    fn default() -> Self {
        Self {
            pieces: [
                [
                    Bitboard::from(squares![b1, g1]),
                    Bitboard::from(squares![c1, f1]),
                    Bitboard::from(squares![a1, h1]),
                    Bitboard::from(square!(d1)),
                    Bitboard::from(squares![a2, b2, c2, d2, e2, f2, g2, h2]),
                    Bitboard::from(square!(e1)),
                ],
                [
                    Bitboard::from(squares![b8, g8]),
                    Bitboard::from(squares![c8, f8]),
                    Bitboard::from(squares![a8, h8]),
                    Bitboard::from(square!(d8)),
                    Bitboard::from(squares![a7, b7, c7, d7, e7, f7, g7, h7]),
                    Bitboard::from(square!(e8)),
                ],
            ],
            mailbox: DEFAULT_MAILBOX,
        }
    }
}

impl Board {
    pub(super) fn from_fen(placement: &str) -> Result<Self> {
        let board = Self::parse_fen_placement(placement)?;

        if board.king::<{ Color::White }>().len() != 1
            || board.king::<{ Color::Black }>().len() != 1
        {
            bail!("FEN position must contain exactly one king of each color");
        }

        Ok(board)
    }

    fn parse_fen_placement(placement: &str) -> Result<Self> {
        let mut board = Self {
            pieces: [[Bitboard::EMPTY; 6]; 2],
            mailbox: [MailboxEntry::EMPTY; 64],
        };

        let ranks = placement.split('/').collect::<Vec<_>>();
        if ranks.len() != 8 {
            bail!("FEN piece placement must contain 8 ranks");
        }

        for (rank_index, rank) in ranks.into_iter().enumerate() {
            let board_rank = 7 - rank_index as u32;
            let mut file = 0;

            for piece in rank.chars() {
                if let Some(empty) = piece.to_digit(10) {
                    if !(1..=8).contains(&empty) {
                        bail!("FEN empty square count must be between 1 and 8");
                    }
                    file += empty;
                    continue;
                }

                if file >= 8 {
                    bail!("FEN rank contains too many squares");
                }

                let square = Square::new(board_rank * 8 + file);
                board.add_piece(piece, square)?;
                file += 1;
            }

            if file != 8 {
                bail!("FEN rank does not contain 8 squares");
            }
        }

        Ok(board)
    }

    #[cfg(test)]
    pub(super) fn from_ascii(ascii: &str) -> Self {
        let mut fen = String::new();
        let squares = ascii
            .chars()
            .filter(|square| !square.is_whitespace())
            .collect::<Vec<_>>();

        assert_eq!(squares.len(), 64, "board must contain 64 squares");

        for (index, square) in squares.into_iter().enumerate() {
            if index > 0 && index % 8 == 0 {
                fen.push('/');
            }

            match square {
                '.' => match fen.pop() {
                    Some(previous) if previous.is_ascii_digit() => {
                        fen.push(char::from(previous as u8 + 1));
                    }
                    Some(previous) => {
                        fen.push(previous);
                        fen.push('1');
                    }
                    None => fen.push('1'),
                },
                piece => fen.push(piece),
            }
        }

        Self::parse_fen_placement(&fen).expect("board! generated invalid FEN")
    }

    pub(super) fn pawns<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Pawn]
    }

    pub(super) fn knights<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Knight]
    }

    pub(super) fn rooks<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Rook]
    }

    pub(super) fn bishops<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Bishop]
    }

    pub(super) fn queens<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Queen]
    }

    pub(super) fn king<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::King]
    }

    pub(super) fn king_square<const COLOR: Color>(&self) -> Square {
        let mut king = self.king::<COLOR>();
        debug_assert_eq!(king.len(), 1);

        unsafe { king.next().unwrap_unchecked() }
    }

    pub(super) fn occupancy<const COLOR: Color>(&self) -> Bitboard {
        self.pieces[COLOR][Piece::Pawn]
            | self.pieces[COLOR][Piece::Rook]
            | self.pieces[COLOR][Piece::Knight]
            | self.pieces[COLOR][Piece::Bishop]
            | self.pieces[COLOR][Piece::Queen]
            | self.pieces[COLOR][Piece::King]
    }

    pub(super) fn add_normal_moves(
        &self,
        list: &mut MoveList,
        from: Square,
        targets: Bitboard,
        enemies: Bitboard,
    ) {
        list.extend((targets & !enemies).map(|to| Move::new(from, to, MoveKind::Quiet)));
        list.extend((targets & enemies).map(|to| Move::new(from, to, MoveKind::Capture)));
    }

    /// # Safety
    ///
    /// `mve` must describe a move valid for the current board and `color`.
    pub(super) unsafe fn make_move(&mut self, color: Color, mve: Move) -> BoardUndo {
        let from = mve.from();
        let to = mve.to();
        let kind = mve.kind();
        let promotion = kind.promotion();
        let source = self.mailbox[from];
        let moved =
            if promotion.is_some() || matches!(kind, MoveKind::DoublePush | MoveKind::EnPassant) {
                Piece::Pawn
            } else if matches!(kind, MoveKind::CastleKing | MoveKind::CastleQueen) {
                Piece::King
            } else {
                debug_assert_eq!(source.color(), Some(color));
                // SAFETY: By `make_move`'s contract, the source entry contains a piece.
                unsafe { source.piece_unchecked() }
            };
        let (captured_square, captured) = match kind {
            MoveKind::EnPassant => {
                let square = match color {
                    Color::White => to.backward::<{ Color::White }, 1>(),
                    Color::Black => to.backward::<{ Color::Black }, 1>(),
                };
                (square, Some(Piece::Pawn))
            }
            kind if kind.is_capture() => {
                let target = self.mailbox[to];
                debug_assert_eq!(target.color(), Some(color.opponent()));
                // SAFETY: By `make_move`'s contract, a capture's target entry contains a piece.
                let piece = unsafe { target.piece_unchecked() };
                (to, Some(piece))
            }
            _ => (to, None),
        };
        if let Some(captured) = captured {
            self.pieces[color.opponent()][captured] &= !Bitboard::from(captured_square);
        }

        self.pieces[color][moved].apply_move(from, to);
        self.mailbox[from] = MailboxEntry::EMPTY;
        self.mailbox[to] = source;

        match (kind, promotion) {
            (MoveKind::EnPassant, _) => {
                self.mailbox[captured_square] = MailboxEntry::EMPTY;
            }
            (MoveKind::CastleKing | MoveKind::CastleQueen, _) => {
                let king_side = usize::from(kind == MoveKind::CastleKing);
                let (rook_from, rook_to) = CASTLING_ROOK_MOVES[color][king_side];

                self.pieces[color][Piece::Rook].apply_move(rook_from, rook_to);
                self.mailbox[rook_from] = MailboxEntry::EMPTY;
                self.mailbox[rook_to] = MailboxEntry::new(color, Piece::Rook);
            }
            (_, Some(promotion)) => {
                debug_assert_eq!(moved, Piece::Pawn);
                self.pieces[color][Piece::Pawn] &= !Bitboard::from(to);
                self.pieces[color][Piece::from(promotion)] |= to;
                self.mailbox[to] = MailboxEntry::new(color, Piece::from(promotion));
            }
            _ => {}
        }

        BoardUndo {
            moved: source,
            captured: captured.map_or(MailboxEntry::EMPTY, |piece| {
                MailboxEntry::new(color.opponent(), piece)
            }),
        }
    }

    /// # Safety
    ///
    /// `undo` must come from the immediately preceding
    /// `make_move(color, mve)` call on this board.
    #[cfg(any(test, feature = "benchmark"))]
    pub(super) fn unmake_move(&mut self, color: Color, mve: Move, undo: BoardUndo) {
        let from = mve.from();
        let to = mve.to();
        let kind = mve.kind();
        let promotion = kind.promotion();
        // SAFETY: `undo` was returned by making this move on the current board.
        let moved = unsafe { undo.moved.piece_unchecked() };

        match (kind, promotion) {
            (MoveKind::CastleKing | MoveKind::CastleQueen, _) => {
                let king_side = usize::from(kind == MoveKind::CastleKing);
                let (rook_from, rook_to) = CASTLING_ROOK_MOVES[color][king_side];
                self.pieces[color][Piece::Rook].apply_move(rook_to, rook_from);

                let rook = self.mailbox[rook_to];
                self.mailbox[rook_to] = MailboxEntry::EMPTY;
                self.mailbox[rook_from] = rook;

                self.pieces[color][moved].apply_move(to, from);
                self.mailbox[from] = self.mailbox[to];
            }
            (_, Some(promotion)) => {
                let promoted = Piece::from(promotion);
                self.pieces[color][promoted] &= !Bitboard::from(to);
                self.pieces[color][Piece::Pawn] |= from;
                self.mailbox[from] = MailboxEntry::new(color, Piece::Pawn);
            }
            _ => {
                self.pieces[color][moved].apply_move(to, from);
                self.mailbox[from] = self.mailbox[to];
            }
        }

        self.mailbox[to] = MailboxEntry::EMPTY;

        if let Some(captured) = undo.captured.piece() {
            let captured_square = match kind {
                MoveKind::EnPassant => match color {
                    Color::White => to.backward::<{ Color::White }, 1>(),
                    Color::Black => to.backward::<{ Color::Black }, 1>(),
                },
                _ => to,
            };
            self.pieces[color.opponent()][captured] |= captured_square;
            self.mailbox[captured_square] = undo.captured;
        }
    }

    pub(super) fn fen(&self) -> String {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty = 0;

            for file in 0..8 {
                let sq = Square::new(rank * 8 + file);
                match self.fen_piece_at(sq) {
                    Some(piece) => {
                        if empty > 0 {
                            fen.push(
                                char::from_digit(empty, 10).expect("empty count fits in digit"),
                            );
                            empty = 0;
                        }
                        fen.push(piece);
                    }
                    None => empty += 1,
                }
            }

            if empty > 0 {
                fen.push(char::from_digit(empty, 10).expect("empty count fits in digit"));
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen
    }

    pub(super) fn occupied(&self) -> Bitboard {
        self.occupancy::<{ Color::White }>() | self.occupancy::<{ Color::Black }>()
    }

    fn add_piece(&mut self, piece: char, square: Square) -> Result<()> {
        let (color, piece) =
            Piece::from_fen(piece).ok_or_else(|| anyhow::anyhow!("invalid FEN piece: {piece}"))?;

        self.pieces[color][piece] |= square;
        self.mailbox[square] = MailboxEntry::new(color, piece);
        Ok(())
    }

    fn fen_piece_at(&self, sq: Square) -> Option<char> {
        self.mailbox[sq]
            .colored_piece()
            .map(|(color, piece)| piece.fen(color))
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "{}  ", rank + 1)?;

            for file in 0..8 {
                let sq = Square::new(rank * 8 + file);
                let piece = self.fen_piece_at(sq).unwrap_or('.');

                write!(f, "{piece}")?;

                if file < 7 {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "\n   A B C D E F G H")
    }
}

impl From<PromotionPiece> for Piece {
    fn from(promotion: PromotionPiece) -> Self {
        // SAFETY: PromotionPiece discriminants align with the first four Piece discriminants.
        unsafe { Self::from_code_unchecked(promotion as u8) }
    }
}

const _: () = {
    assert!(Piece::Knight as u8 == PromotionPiece::Knight as u8);
    assert!(Piece::Bishop as u8 == PromotionPiece::Bishop as u8);
    assert!(Piece::Rook as u8 == PromotionPiece::Rook as u8);
    assert!(Piece::Queen as u8 == PromotionPiece::Queen as u8);
    assert!(Piece::ALL.len() as u8 <= MailboxEntry::EMPTY.0);
};

const impl<T> Index<Piece> for [T; 6] {
    type Output = T;

    fn index(&self, piece: Piece) -> &Self::Output {
        &self[piece as usize]
    }
}

const impl<T> IndexMut<Piece> for [T; 6] {
    fn index_mut(&mut self, piece: Piece) -> &mut Self::Output {
        &mut self[piece as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, Piece, Square};
    use crate::{
        moves::{Move, MoveKind},
        square,
        state::Color,
    };

    fn assert_mailbox_consistent(board: &Board) {
        use crate::bitboard::Bitboard;

        for square in Bitboard::FULL {
            let scanned = Color::ALL.into_iter().find_map(|color| {
                Piece::ALL
                    .into_iter()
                    .find(|piece| board.pieces[color][*piece].contains(square))
                    .map(|piece| (color, piece))
            });
            assert_eq!(board.mailbox[square].colored_piece(), scanned);
        }
    }

    fn assert_mailbox_move_roundtrip(
        mut board: Board,
        color: Color,
        from: Square,
        to: Square,
        kind: MoveKind,
    ) {
        let initial_fen = board.fen();
        let mve = Move::new(from, to, kind);
        assert_mailbox_consistent(&board);
        // SAFETY: Each fixture describes a move valid for its board and color.
        let undo = unsafe { board.make_move(color, mve) };
        assert_mailbox_consistent(&board);
        board.unmake_move(color, mve, undo);
        assert_mailbox_consistent(&board);
        assert_eq!(board.fen(), initial_fen);
    }

    #[test]
    fn default_board_fen() {
        assert_eq!(
            Board::default().fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        );
    }

    #[test]
    fn board_roundtrips_through_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

        assert_eq!(Board::from_fen(fen).unwrap().fen(), fen);
    }

    #[test]
    fn parses_mixed_piece_fen() {
        let fen = "8/3k4/2p5/8/4P3/8/3K4/8";

        assert_eq!(Board::from_fen(fen).unwrap().fen(), fen);
    }

    #[test]
    fn rejects_invalid_fen_placements() {
        for fen in [
            "8/8/8/8/8/8/8",
            "8/8/8/8/8/8/8/8/8",
            "8/8/8/8/8/8/8/7",
            "8/8/8/8/8/8/8/9",
            "8/8/8/8/8/8/8/0",
            "8/8/8/8/8/8/8/7X",
            "8/8/8/8/8/8/8/8P",
            "8/8/8/8/8/8/8/8",
            "7k/8/8/8/8/8/8/8",
            "8/8/8/8/8/8/8/4K3",
            "k6k/8/8/8/8/8/8/4K3",
            "7k/8/8/8/8/8/8/K6K",
        ] {
            assert!(Board::from_fen(fen).is_err(), "{fen} should be invalid");
        }
    }

    #[test]
    fn mailbox_stays_consistent_for_all_move_kinds_and_unmake() {
        for (board, color, from, to, kind) in [
            (
                Board::from_fen("7k/8/8/8/8/8/4P3/K7").unwrap(),
                Color::White,
                square!(e2),
                square!(e3),
                MoveKind::Quiet,
            ),
            (
                Board::default(),
                Color::White,
                square!(e2),
                square!(e4),
                MoveKind::DoublePush,
            ),
            (
                Board::from_fen("7k/8/8/3p4/4P3/8/8/K7").unwrap(),
                Color::White,
                square!(e4),
                square!(d5),
                MoveKind::Capture,
            ),
            (
                Board::from_fen("7k/8/8/3pP3/8/8/8/K7").unwrap(),
                Color::White,
                square!(e5),
                square!(d6),
                MoveKind::EnPassant,
            ),
            (
                Board::from_fen("4k3/8/8/8/8/8/8/R3K2R").unwrap(),
                Color::White,
                square!(e1),
                square!(g1),
                MoveKind::CastleKing,
            ),
            (
                Board::from_fen("4k3/8/8/8/8/8/8/R3K2R").unwrap(),
                Color::White,
                square!(e1),
                square!(c1),
                MoveKind::CastleQueen,
            ),
            (
                Board::from_fen("7k/P7/8/8/8/8/8/K7").unwrap(),
                Color::White,
                square!(a7),
                square!(a8),
                MoveKind::PromoteKnight,
            ),
            (
                Board::from_fen("1r5k/P7/8/8/8/8/8/K7").unwrap(),
                Color::White,
                square!(a7),
                square!(b8),
                MoveKind::CapturePromoteQueen,
            ),
        ] {
            assert_mailbox_move_roundtrip(board, color, from, to, kind);
        }
    }
}
