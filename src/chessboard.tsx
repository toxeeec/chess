import { Modifier, type DragOperation } from "@dnd-kit/abstract"
import { RestrictToElement } from "@dnd-kit/dom/modifiers"
import { DragDropProvider, useDraggable, useDroppable } from "@dnd-kit/react"
import { useRef, useSyncExternalStore } from "react"

import { clsx } from "#/clsx"

const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"] as const
const RANKS = [8, 7, 6, 5, 4, 3, 2, 1] as const

const Square = {
	getFile(square: number) {
		return square % 8
	},
	getRank(square: number) {
		return Math.floor(square / 8)
	},
	isLight(square: number) {
		const rank = this.getRank(square)
		const file = this.getFile(square)
		return (rank + file) % 2 === 0
	},
}

type Piece = "r" | "n" | "b" | "q" | "k" | "p" | "R" | "N" | "B" | "Q" | "K" | "P"

const INITIAL_BOARD = (() => {
	const board: (Piece | undefined)[] = Array.from({ length: 64 })

	board[0] = "r"
	board[1] = "n"
	board[2] = "b"
	board[3] = "q"
	board[4] = "k"
	board[5] = "b"
	board[6] = "n"
	board[7] = "r"

	board[56] = "R"
	board[57] = "N"
	board[58] = "B"
	board[59] = "Q"
	board[60] = "K"
	board[61] = "B"
	board[62] = "N"
	board[63] = "R"

	for (let file = 0; file < 8; file += 1) {
		board[8 + file] = "p"

		board[48 + file] = "P"
	}

	return board
})()

let board = INITIAL_BOARD
const boardListeners = new Set<() => void>()

function subscribeToBoard(listener: () => void) {
	boardListeners.add(listener)
	return () => boardListeners.delete(listener)
}

function emitBoardChange() {
	for (const listener of boardListeners) {
		listener()
	}
}

function getPieceOnSquare(square: number) {
	return board[square]
}

function movePiece(sourceSquare: number, targetSquare: number) {
	const movingPiece = board[sourceSquare]
	if (!movingPiece || sourceSquare === targetSquare) return

	board = [...board]
	board[sourceSquare] = undefined
	board[targetSquare] = movingPiece
	emitBoardChange()
}

class SnapToPointer extends Modifier {
	apply({ activatorEvent, transform, shape }: DragOperation) {
		// `shape` is null on the first call. Remove this guard once https://github.com/clauderic/dnd-kit/pull/1988 lands.
		if (!shape || !(activatorEvent instanceof PointerEvent)) {
			return transform
		}

		const rect = shape.initial.boundingRectangle
		const anchorX = rect.left + rect.width * 0.5
		const anchorY = rect.top + rect.height * 0.5

		return {
			x: transform.x + activatorEvent.clientX - anchorX,
			y: transform.y + activatorEvent.clientY - anchorY,
		}
	}
}

export function Chessboard() {
	const ref = useRef<HTMLDivElement>(null)
	return (
		<DragDropProvider
			modifiers={[
				SnapToPointer,
				RestrictToElement.configure({
					element: () => ref.current,
				}),
			]}
			onDragEnd={({ operation: { source, target } }) => {
				if (!source || !target) return
				movePiece(Number(source.id), Number(target.id))
			}}
		>
			<div className="relative grid size-[round(down,80vmin,8px)] grid-cols-8 justify-self-center">
				<div ref={ref} className="absolute inset-[-6.25%]" />
				{Array.from({ length: 64 }, (_, index) => index).map((square) => (
					<BoardSquare key={square} square={square} />
				))}
			</div>
		</DragDropProvider>
	)
}

function BoardSquare({ square }: { square: number }) {
	const piece = useSyncExternalStore(
		subscribeToBoard,
		() => getPieceOnSquare(square),
		() => getPieceOnSquare(square),
	)
	const { isDropTarget, ref } = useDroppable({
		id: square,
	})
	const light = Square.isLight(square)

	return (
		<div
			ref={ref}
			className={clsx(
				"relative grid place-items-center",
				light ? "bg-neutral-300 inset-ring-neutral-400" : "bg-neutral-800 inset-ring-neutral-400",
				isDropTarget && "inset-ring-[0.5vmin]",
			)}
		>
			{Square.getFile(square) === 0 && <Coordinate square={square} rank />}
			{Square.getRank(square) === 7 && <Coordinate square={square} file />}
			{piece !== undefined && <DraggablePiece piece={piece} square={square} />}
		</div>
	)
}

function Coordinate({
	square,
	rank,
	file,
}: { square: number } & ({ rank: true; file?: never } | { file: true; rank?: never })) {
	return (
		<span
			className={clsx(
				"absolute text-[2.2vmin] leading-none font-semibold select-none",
				Square.isLight(square) ? "text-neutral-800" : "text-neutral-300",
				rank && "top-[6%] left-[6%]",
				file && "right-[6%] bottom-[6%]",
			)}
		>
			{rank && RANKS[Square.getRank(square)]}
			{file && FILES[Square.getFile(square)]}
		</span>
	)
}

function DraggablePiece({ piece, square }: { piece: Piece; square: number }) {
	const { ref, handleRef } = useDraggable({ id: square })

	return (
		<button
			ref={(element) => {
				ref(element)
				handleRef(element)
			}}
			className="absolute z-10 size-full cursor-grab touch-none text-[3vmin] leading-none font-bold text-stone-100 select-none active:cursor-grabbing"
			type="button"
		>
			{piece}
		</button>
	)
}
