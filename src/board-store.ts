import { createContext, use, useSyncExternalStore } from "react"

import type { Move } from "./use-live-room"

type BoardStore = ReturnType<typeof createBoardStore>
type BoardState = ReturnType<BoardStore["getState"]>
type PendingPromotion = { from: number; to: number }

export const BoardStoreContext = createContext<BoardStore | null>(null)

export function createBoardStore({
	onMove,
	getLegalMoves,
}: {
	onMove: (move: Move) => void
	getLegalMoves: () => readonly Move[]
}) {
	const listeners = new Set<() => void>()
	const notify = () => {
		for (const listener of listeners) {
			listener()
		}
	}

	const setPendingPromotion = (pendingPromotion: PendingPromotion | null) => {
		if (state.pendingPromotion === pendingPromotion) return

		state = { ...state, pendingPromotion }
		notify()
	}

	const requestMove = (move: Move) => {
		const requiresPromotion =
			!move.promotion &&
			getLegalMoves().some(
				(legalMove) =>
					legalMove.from === move.from && legalMove.to === move.to && !!legalMove.promotion,
			)

		if (requiresPromotion) {
			setPendingPromotion({ from: move.from, to: move.to })
		} else {
			onMove(move)
			setPendingPromotion(null)
		}
	}

	let state = {
		draggedPieceSquare: null as number | null,
		pendingPromotion: null as PendingPromotion | null,
		requestMove,
		setPendingPromotion,
	}

	return {
		getState: () => state,
		setDraggedPieceSquare: (draggedPieceSquare: number | null) => {
			if (state.draggedPieceSquare === draggedPieceSquare) return

			state = { ...state, draggedPieceSquare }
			notify()
		},
		requestMove,
		setPendingPromotion,
		subscribe: (listener: () => void) => {
			listeners.add(listener)
			return () => listeners.delete(listener)
		},
	}
}

export function useBoardStore<T>(selector: (state: BoardState) => T) {
	const store = use(BoardStoreContext)
	if (!store) throw new Error("useBoardStore must be used within BoardStoreContext")

	return useSyncExternalStore(
		store.subscribe,
		() => selector(store.getState()),
		() => selector(store.getState()),
	)
}

if (import.meta.vitest) {
	const { expect, it, vi } = import.meta.vitest

	it.concurrent("requests ordinary moves immediately", () => {
		const onMove = vi.fn()
		const store = createBoardStore({ onMove, getLegalMoves: () => [{ from: 52, to: 44 }] })

		store.requestMove({ from: 52, to: 44 })

		expect(onMove).toHaveBeenCalledExactlyOnceWith({ from: 52, to: 44 })
		expect(store.getState().pendingPromotion).toBeNull()
	})

	it.concurrent("defers promotion moves until a piece is selected", () => {
		const onMove = vi.fn()
		const store = createBoardStore({
			onMove,
			getLegalMoves: () => [{ from: 8, to: 0, promotion: "q" }],
		})

		store.requestMove({ from: 8, to: 0 })

		expect(onMove).not.toHaveBeenCalled()
		expect(store.getState().pendingPromotion).toEqual({ from: 8, to: 0 })

		store.requestMove({ from: 8, to: 0, promotion: "q" })

		expect(onMove).toHaveBeenCalledExactlyOnceWith({ from: 8, to: 0, promotion: "q" })
		expect(store.getState().pendingPromotion).toBeNull()
	})
}
