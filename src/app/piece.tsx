import { twMerge } from "@/utils"
import { useDraggable } from "@dnd-kit/core"
import { CSS } from "@dnd-kit/utilities"
import { ComponentProps } from "react"

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

export const FILES = ["A", "B", "C", "D", "E", "F", "G", "H"] as const

type PieceType = keyof typeof PIECE_TYPE
type PieceColor = keyof typeof PIECE_COLOR
type File = (typeof FILES)[number]
type Rank = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
type Square = `${File}${Rank}`

export function Piece({
	type,
	color,
	square,
}: {
	type: PieceType
	color: PieceColor
	square: Square
}) {
	const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
		id: square,
	})

	return (
		<button
			{...listeners}
			{...attributes}
			className={twMerge(isDragging && "cursor-grabbing")}
			ref={setNodeRef}
			style={{
				...getPieceStyle(type, color, square),
				transform: CSS.Translate.toString(transform),
			}}
		/>
	)
}

function getPieceStyle(type: PieceType, color: PieceColor, square: Square) {
	const backgroundImage = `url(/${PIECE_COLOR[color]}-${PIECE_TYPE[type]}.svg)`
	const gridColumn = square.charCodeAt(0) - "A".charCodeAt(0) + 1
	const gridRow = 8 - parseInt(square[1] as string) + 1

	return { backgroundImage, gridColumn, gridRow } satisfies ComponentProps<"div">["style"]
}
