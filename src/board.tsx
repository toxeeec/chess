import { Modifier, type DragOperation } from "@dnd-kit/abstract"
import { RestrictToElement } from "@dnd-kit/dom/modifiers"
import { DragDropProvider, useDraggable, useDroppable } from "@dnd-kit/react"
import { createContext, use, useRef, useState, useSyncExternalStore } from "react"

import { clsx } from "./clsx"
import { useGameStore, useShallow, type Piece } from "./game-store"
import type { Move } from "./use-live-room"

const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"] as const
const RANKS = [8, 7, 6, 5, 4, 3, 2, 1] as const

type BoardStore = ReturnType<typeof createBoardStore>
type BoardState = ReturnType<BoardStore["getState"]>

const BoardStoreContext = createContext<BoardStore | null>(null)

export function Board({ onMove }: { onMove: (move: Move) => void }) {
	const ref = useRef<HTMLDivElement>(null)
	const [boardStore] = useState(createBoardStore)

	return (
		<BoardStoreContext value={boardStore}>
			<DragDropProvider
				modifiers={[
					SnapToPointer,
					// oxlint-disable-next-line react/react-compiler
					RestrictToElement.configure({
						element: () => ref.current,
					}),
				]}
				onDragStart={({ operation: { source } }) => {
					boardStore.setDraggedPieceSquare(source ? Number(source.id) : null)
				}}
				onDragEnd={({ operation: { source, target } }) => {
					if (source && target) onMove({ from: Number(source.id), to: Number(target.id) })
					boardStore.setDraggedPieceSquare(null)
				}}
			>
				<div className="relative grid size-[round(down,80vmin,8px)] grid-cols-8 justify-self-center">
					<div ref={ref} className="absolute inset-[-6.25%] -z-10" />
					{Array.from({ length: 64 }, (_, index) => index).map((square) => (
						<BoardSquare key={square} square={square} />
					))}
				</div>
			</DragDropProvider>
		</BoardStoreContext>
	)
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

function BoardSquare({ square }: { square: number }) {
	const [piece, isLegalMoveTarget, disabled] = useGameStore(
		useShallow((store) => [
			store.board[square],
			store.legalMoves.some(({ to }) => to === square),
			!store.legalMoves.some(({ from }) => from === square),
		]),
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
			{isLegalMoveTarget && <LegalMoveDot square={square} />}
			{piece && <DraggablePiece piece={piece} square={square} disabled={disabled} />}
		</div>
	)
}

function createBoardStore() {
	let state = { draggedPieceSquare: null as number | null }
	const listeners = new Set<() => void>()
	const notify = () => {
		for (const listener of listeners) {
			listener()
		}
	}

	return {
		getState: () => state,
		setDraggedPieceSquare: (draggedPieceSquare: number | null) => {
			if (state.draggedPieceSquare === draggedPieceSquare) return

			state = { draggedPieceSquare }
			notify()
		},
		subscribe: (listener: () => void) => {
			listeners.add(listener)
			return () => listeners.delete(listener)
		},
	}
}

function useBoardStore<T>(selector: (state: BoardState) => T) {
	const store = use(BoardStoreContext)
	if (!store) throw new Error("useBoardStore must be used within BoardStoreContext")

	return useSyncExternalStore(
		store.subscribe,
		() => selector(store.getState()),
		() => selector(store.getState()),
	)
}

function LegalMoveDot({ square }: { square: number }) {
	const legalMoves = useGameStore((store) => store.legalMoves)
	const visible = useBoardStore(
		(store) =>
			store.draggedPieceSquare !== null &&
			legalMoves.some(({ from, to }) => from === store.draggedPieceSquare && to === square),
	)

	return (
		<span
			className={clsx(
				"pointer-events-none invisible absolute size-1/4 rounded-full bg-neutral-400",
				visible && "visible",
			)}
		/>
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

function DraggablePiece({
	piece,
	square,
	disabled,
}: {
	piece: Piece
	square: number
	disabled: boolean
}) {
	const { ref, handleRef } = useDraggable({ id: square, disabled })

	return (
		<button
			ref={(element) => {
				ref(element)
				handleRef(element)
			}}
			className={clsx(
				"absolute z-10 size-full touch-none text-[3vmin] leading-none font-bold text-stone-100 select-none",
				!disabled && "cursor-grab active:cursor-grabbing",
			)}
		>
			{piece}
		</button>
	)
}
