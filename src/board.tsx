import { Modifier, type DragOperation } from "@dnd-kit/abstract"
import { RestrictToElement } from "@dnd-kit/dom/modifiers"
import { DragDropProvider } from "@dnd-kit/react"
import { useRef, useState } from "react"

import { BoardStoreContext, createBoardStore } from "./board-store"
import { useGameStore } from "./game-store"
import { PromotionDialog } from "./piece"
import { BoardSquare } from "./square"
import type { Move } from "./use-live-room"

export function Board({ onMove }: { onMove: (move: Move) => void }) {
	const ref = useRef<HTMLDivElement>(null)
	const getLegalMoves = useGameStore((store) => store.getLegalMoves)
	const [boardStore] = useState(() => createBoardStore({ onMove, getLegalMoves }))

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
					boardStore.setDraggedPieceSquare(null)
					if (!source || !target) return
					boardStore.requestMove({ from: Number(source.id), to: Number(target.id) })
				}}
			>
				<div className="relative grid size-[round(down,80vmin,8px)] grid-cols-8 justify-self-center">
					<div ref={ref} className="absolute inset-[-6.25%] -z-10" />
					{Array.from({ length: 64 }, (_, index) => index).map((square) => (
						<BoardSquare key={square} square={square} />
					))}
					<PromotionDialog />
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
