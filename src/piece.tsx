import { useDraggable } from "@dnd-kit/react"
import { useEffect, useRef } from "react"

import { useBoardStore } from "./board-store"
import { clsx } from "./clsx"
import { useGameStore } from "./game-store"
import { Square } from "./square"
import { useShallow } from "./store"
import type { Move } from "./use-live-room"

const PIECES = ["r", "n", "b", "q", "k", "p", "R", "N", "B", "Q", "K", "P"] as const
export type Piece = (typeof PIECES)[number]

export const Piece = {
	is(piece: string): piece is Piece {
		return PIECES.includes(piece)
	},
	promote(piece: Piece, promotion: NonNullable<Move["promotion"]>) {
		return piece === piece.toUpperCase()
			? // oxlint-disable-next-line typescript/no-unsafe-type-assertion
				(promotion.toUpperCase() as Uppercase<typeof promotion>)
			: promotion
	},
} as const

export function DraggablePiece({
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
				"absolute z-10 size-full touch-none text-[3vmin] leading-none font-bold text-neutral-100 select-none",
				!disabled && "cursor-grab active:cursor-grabbing",
			)}
		>
			{piece}
		</button>
	)
}

export function PromotionDialog() {
	const ref = useRef<HTMLDialogElement>(null)
	const [pendingPromotion, setPendingPromotion, requestMove] = useBoardStore(
		useShallow((store) => [store.pendingPromotion, store.setPendingPromotion, store.requestMove]),
	)
	const [promotingPiece, ...promotions] = useGameStore(
		useShallow((store) => [
			pendingPromotion ? store.board[pendingPromotion.from] : undefined,
			...store.legalMoves
				.filter(
					(move) =>
						move.from === pendingPromotion?.from &&
						move.to === pendingPromotion.to &&
						move.promotion,
				)
				.map((move) => move.promotion!),
		]),
	)

	useEffect(() => {
		const dialog = ref.current
		if (!dialog) return

		if (pendingPromotion && !dialog.open) {
			dialog.showModal()
		} else if (!pendingPromotion && dialog.open) {
			dialog.close()
		}
	}, [pendingPromotion])

	return (
		<dialog
			ref={ref}
			closedby="any"
			className={clsx(
				"inset-auto left-[anchor(left)] flex w-[anchor-size(width)] bg-neutral-200 text-neutral-800 [position-anchor:--promotion-square] backdrop:bg-neutral-950/35",
				!!pendingPromotion &&
					(Square.rank(pendingPromotion.to) === 0
						? "top-[anchor(top)] flex-col"
						: "bottom-[anchor(bottom)] flex-col-reverse"),
			)}
			onClose={() => setPendingPromotion(null)}
		>
			{pendingPromotion && promotingPiece && (
				<>
					{promotions.map((promotion) => (
						<button
							key={promotion}
							type="button"
							className="grid aspect-square cursor-pointer place-items-center text-[3vmin] leading-none font-bold hover:bg-neutral-300/60"
							onClick={() => requestMove({ ...pendingPromotion, promotion })}
						>
							{Piece.promote(promotingPiece, promotion)}
						</button>
					))}
					<button
						type="button"
						className="grid aspect-2/1 cursor-pointer place-items-center bg-neutral-400/30 text-[2.5vmin] leading-none text-neutral-600/80 hover:bg-neutral-400/50 hover:text-neutral-600"
						onClick={() => setPendingPromotion(null)}
					>
						×
					</button>
				</>
			)}
		</dialog>
	)
}
