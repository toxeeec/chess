import { Piece } from "./piece"
import { Square } from "./square"
import { useState } from "react"

type Position = Partial<Record<Square, Piece>>

export function useChessState() {
	const [position, setPosition] = useState<Position>(STARTING_POSITION)
	const pieces = Object.entries(position) as ReadonlyArray<readonly [Square, Piece]>

	const movePiece = (fromSq: Square, toSq: Square) => {
		if (fromSq === toSq) return
		setPosition((prevPos) => {
			const from = prevPos[fromSq]
			if (!from) return prevPos

			delete prevPos[fromSq]
			return { ...prevPos, [toSq]: from }
		})
	}

	return { pieces, movePiece }
}

const STARTING_POSITION = {
	A1: { type: "ROOK", color: "WHITE" },
	B1: { type: "KNIGHT", color: "WHITE" },
	C1: { type: "BISHOP", color: "WHITE" },
	D1: { type: "QUEEN", color: "WHITE" },
	E1: { type: "KING", color: "WHITE" },
	F1: { type: "BISHOP", color: "WHITE" },
	G1: { type: "KNIGHT", color: "WHITE" },
	H1: { type: "ROOK", color: "WHITE" },
	A2: { type: "PAWN", color: "WHITE" },
	B2: { type: "PAWN", color: "WHITE" },
	C2: { type: "PAWN", color: "WHITE" },
	D2: { type: "PAWN", color: "WHITE" },
	E2: { type: "PAWN", color: "WHITE" },
	F2: { type: "PAWN", color: "WHITE" },
	G2: { type: "PAWN", color: "WHITE" },
	H2: { type: "PAWN", color: "WHITE" },
	A7: { type: "PAWN", color: "BLACK" },
	B7: { type: "PAWN", color: "BLACK" },
	C7: { type: "PAWN", color: "BLACK" },
	D7: { type: "PAWN", color: "BLACK" },
	E7: { type: "PAWN", color: "BLACK" },
	F7: { type: "PAWN", color: "BLACK" },
	G7: { type: "PAWN", color: "BLACK" },
	H7: { type: "PAWN", color: "BLACK" },
	A8: { type: "ROOK", color: "BLACK" },
	B8: { type: "KNIGHT", color: "BLACK" },
	C8: { type: "BISHOP", color: "BLACK" },
	D8: { type: "QUEEN", color: "BLACK" },
	E8: { type: "KING", color: "BLACK" },
	F8: { type: "BISHOP", color: "BLACK" },
	G8: { type: "KNIGHT", color: "BLACK" },
	H8: { type: "ROOK", color: "BLACK" },
} as const satisfies Position
