const PIECE_TYPE = {
	PAWN: "pawn",
	ROOK: "rook",
	KNIGHT: "knight",
	BISHOP: "bishop",
	QUEEN: "queen",
	KING: "king",
} as const

const PIECE_COLOR = {
	WHITE: "white",
	BLACK: "black",
} as const

type PieceType = keyof typeof PIECE_TYPE
type PieceColor = keyof typeof PIECE_COLOR
export type Piece = { type: PieceType; color: PieceColor }

export namespace Piece {
	export function imagePath(piece: Piece) {
		return `/${PIECE_COLOR[piece.color]}-${PIECE_TYPE[piece.type]}.svg`
	}
}
