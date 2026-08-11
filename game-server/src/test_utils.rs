use crate::{board::Board, moves::MoveList};

macro_rules! board {
    (@square .) =>  { "." };
    (@square P) => { "P" };
    (@square N) => { "N" };
    (@square B) => { "B" };
    (@square R) => { "R" };
    (@square Q) => { "Q" };
    (@square K) => { "K" };
    (@square p) => { "p" };
    (@square n) => { "n" };
    (@square b) => { "b" };
    (@square r) => { "r" };
    (@square q) => { "q" };
    (@square k) => { "k" };
    (@square $invalid:tt) => {
        compile_error!(concat!(
            "invalid board square `",
            stringify!($invalid),
            "`; expected one of . P N B R Q K p n b r q k"
        ))
    };
    (
        $a8:tt $b8:tt $c8:tt $d8:tt $e8:tt $f8:tt $g8:tt $h8:tt
        $a7:tt $b7:tt $c7:tt $d7:tt $e7:tt $f7:tt $g7:tt $h7:tt
        $a6:tt $b6:tt $c6:tt $d6:tt $e6:tt $f6:tt $g6:tt $h6:tt
        $a5:tt $b5:tt $c5:tt $d5:tt $e5:tt $f5:tt $g5:tt $h5:tt
        $a4:tt $b4:tt $c4:tt $d4:tt $e4:tt $f4:tt $g4:tt $h4:tt
        $a3:tt $b3:tt $c3:tt $d3:tt $e3:tt $f3:tt $g3:tt $h3:tt
        $a2:tt $b2:tt $c2:tt $d2:tt $e2:tt $f2:tt $g2:tt $h2:tt
        $a1:tt $b1:tt $c1:tt $d1:tt $e1:tt $f1:tt $g1:tt $h1:tt
    ) => {
        crate::board::Board::from_ascii(concat!(
            board!(@square $a8), board!(@square $b8), board!(@square $c8), board!(@square $d8),
            board!(@square $e8), board!(@square $f8), board!(@square $g8), board!(@square $h8),
            board!(@square $a7), board!(@square $b7), board!(@square $c7), board!(@square $d7),
            board!(@square $e7), board!(@square $f7), board!(@square $g7), board!(@square $h7),
            board!(@square $a6), board!(@square $b6), board!(@square $c6), board!(@square $d6),
            board!(@square $e6), board!(@square $f6), board!(@square $g6), board!(@square $h6),
            board!(@square $a5), board!(@square $b5), board!(@square $c5), board!(@square $d5),
            board!(@square $e5), board!(@square $f5), board!(@square $g5), board!(@square $h5),
            board!(@square $a4), board!(@square $b4), board!(@square $c4), board!(@square $d4),
            board!(@square $e4), board!(@square $f4), board!(@square $g4), board!(@square $h4),
            board!(@square $a3), board!(@square $b3), board!(@square $c3), board!(@square $d3),
            board!(@square $e3), board!(@square $f3), board!(@square $g3), board!(@square $h3),
            board!(@square $a2), board!(@square $b2), board!(@square $c2), board!(@square $d2),
            board!(@square $e2), board!(@square $f2), board!(@square $g2), board!(@square $h2),
            board!(@square $a1), board!(@square $b1), board!(@square $c1), board!(@square $d1),
            board!(@square $e1), board!(@square $f1), board!(@square $g1), board!(@square $h1),
        ))
    };
    ($($invalid:tt)*) => {
        compile_error!(
            "board! expects exactly 64 squares using tokens: . P N B R Q K p n b r q k"
        )
    };
}

pub(super) use board;

macro_rules! moves {
    (@square .) => { "." };
    (@square o) => { "o" };
    (@square x) => { "x" };
    (@square $invalid:tt) => {
        compile_error!(concat!(
            "invalid move square `",
            stringify!($invalid),
            "`; expected one of . o x"
        ))
    };
    (
        $a8:tt $b8:tt $c8:tt $d8:tt $e8:tt $f8:tt $g8:tt $h8:tt
        $a7:tt $b7:tt $c7:tt $d7:tt $e7:tt $f7:tt $g7:tt $h7:tt
        $a6:tt $b6:tt $c6:tt $d6:tt $e6:tt $f6:tt $g6:tt $h6:tt
        $a5:tt $b5:tt $c5:tt $d5:tt $e5:tt $f5:tt $g5:tt $h5:tt
        $a4:tt $b4:tt $c4:tt $d4:tt $e4:tt $f4:tt $g4:tt $h4:tt
        $a3:tt $b3:tt $c3:tt $d3:tt $e3:tt $f3:tt $g3:tt $h3:tt
        $a2:tt $b2:tt $c2:tt $d2:tt $e2:tt $f2:tt $g2:tt $h2:tt
        $a1:tt $b1:tt $c1:tt $d1:tt $e1:tt $f1:tt $g1:tt $h1:tt
    ) => {
        crate::moves::MoveList::from_ascii(concat!(
            moves!(@square $a8), moves!(@square $b8), moves!(@square $c8), moves!(@square $d8),
            moves!(@square $e8), moves!(@square $f8), moves!(@square $g8), moves!(@square $h8),
            moves!(@square $a7), moves!(@square $b7), moves!(@square $c7), moves!(@square $d7),
            moves!(@square $e7), moves!(@square $f7), moves!(@square $g7), moves!(@square $h7),
            moves!(@square $a6), moves!(@square $b6), moves!(@square $c6), moves!(@square $d6),
            moves!(@square $e6), moves!(@square $f6), moves!(@square $g6), moves!(@square $h6),
            moves!(@square $a5), moves!(@square $b5), moves!(@square $c5), moves!(@square $d5),
            moves!(@square $e5), moves!(@square $f5), moves!(@square $g5), moves!(@square $h5),
            moves!(@square $a4), moves!(@square $b4), moves!(@square $c4), moves!(@square $d4),
            moves!(@square $e4), moves!(@square $f4), moves!(@square $g4), moves!(@square $h4),
            moves!(@square $a3), moves!(@square $b3), moves!(@square $c3), moves!(@square $d3),
            moves!(@square $e3), moves!(@square $f3), moves!(@square $g3), moves!(@square $h3),
            moves!(@square $a2), moves!(@square $b2), moves!(@square $c2), moves!(@square $d2),
            moves!(@square $e2), moves!(@square $f2), moves!(@square $g2), moves!(@square $h2),
            moves!(@square $a1), moves!(@square $b1), moves!(@square $c1), moves!(@square $d1),
            moves!(@square $e1), moves!(@square $f1), moves!(@square $g1), moves!(@square $h1),
        ))
    };
    ($($invalid:tt)*) => {
        compile_error!("moves! expects exactly 64 squares using tokens: . o x")
    };
}

pub(super) use moves;

pub(super) struct MoveCase {
    pub(super) name: &'static str,
    pub(super) board: Board,
    pub(super) moves: MoveList,
}

pub(super) fn assert_move_cases<const N: usize>(
    cases: [MoveCase; N],
    generate_moves: impl Fn(Board) -> MoveList,
) {
    for case in cases {
        let moves = generate_moves(case.board);

        assert_eq!(
            moves.len(),
            case.moves.len(),
            "{}: wrong move count; got {moves}",
            case.name
        );

        for expected in case.moves.iter() {
            assert!(
                moves.contains(expected),
                "{}: missing move {expected}; got {moves}",
                case.name
            );
        }
    }
}
