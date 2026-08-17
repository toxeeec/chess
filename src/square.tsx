import { useDroppable } from "@dnd-kit/react"

import { useBoardStore } from "./board-store"
import { clsx } from "./clsx"
import { useGameStore } from "./game-store"
import { DraggablePiece } from "./piece"
import { useShallow } from "./store"

const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"] as const
const RANKS = [8, 7, 6, 5, 4, 3, 2, 1] as const

export const Square = {
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
} as const

export function BoardSquare({ square }: { square: number }) {
	const [pieceHidden, isPromotionTarget] = useBoardStore(
		useShallow((store) => [
			store.pendingPromotion?.from === square,
			store.pendingPromotion?.to === square,
		]),
	)
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
				isPromotionTarget && "[anchor-name:--promotion-square]",
			)}
		>
			{Square.getFile(square) === 0 && <Coordinate square={square} rank />}
			{Square.getRank(square) === 7 && <Coordinate square={square} file />}
			{isLegalMoveTarget && <LegalMoveDot square={square} />}
			{piece && !pieceHidden && (
				<DraggablePiece piece={piece} square={square} disabled={disabled} />
			)}
		</div>
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
