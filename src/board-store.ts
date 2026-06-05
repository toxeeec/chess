import { createContext, use, useSyncExternalStore } from "react"

import type { Player } from "./room"
import type { Move } from "./use-live-room"

const PIECES = ["r", "n", "b", "q", "k", "p", "R", "N", "B", "Q", "K", "P"] as const
export type Piece = (typeof PIECES)[number]

export type BoardStore = ReturnType<typeof createBoardStore>
type BoardState = ReturnType<BoardStore["getState"]>
type Snapshot = Parameters<typeof createBoardState>[0]

export function createBoardStore({
	snapshot,
	player,
	onMove,
}: {
	snapshot: Snapshot
	player: Player
	onMove?: (move: Move) => void
}) {
	let state = createBoardState(snapshot, player)
	const listeners = new Set<() => void>()
	const notify = () => {
		for (const listener of listeners) {
			listener()
		}
	}

	return {
		getState: () => state,
		setState: (snapshot: Snapshot) => {
			state = createBoardState(snapshot, player)
			notify()
		},
		setDraggedPieceSquare: (draggedPieceSquare: number | null) => {
			if (state.draggedPieceSquare === draggedPieceSquare) return

			state = { ...state, draggedPieceSquare }
			notify()
		},
		movePiece: (move: Move) => {
			const movingPiece = state.board[move.from]
			if (!movingPiece || move.from === move.to) return
			if (!state.legalMoves.some(({ from, to }) => from === move.from && to === move.to)) return

			const board = [...state.board]
			board[move.from] = undefined
			board[move.to] = movingPiece
			state = {
				...state,
				board,
				turn: opponent(state.turn),
				legalMoves: [],
				draggedPieceSquare: null,
			}

			onMove?.(move)
			notify()
		},
		applyMove: ({
			move,
			legalMoves,
			turn,
		}: {
			move: Move
			legalMoves: readonly Move[]
			turn: Player
		}) => {
			const movingPiece = state.board[move.from]
			if (!movingPiece) return

			const board = [...state.board]
			board[move.from] = undefined
			board[move.to] = movingPiece
			state = {
				...state,
				board,
				turn,
				legalMoves: turn === player ? legalMoves : [],
			}
			notify()
		},
		subscribe: (listener: () => void) => {
			listeners.add(listener)
			return () => listeners.delete(listener)
		},
	}
}

export const BoardStoreContext = createContext<BoardStore | null>(null)

function createBoardState(
	{
		fen,
		legalMoves,
	}: {
		fen: string
		legalMoves: readonly Move[]
	},
	player: Player,
) {
	const turn = getTurnFromFen(fen)
	return {
		board: createBoardFromFen(fen),
		draggedPieceSquare: null as number | null,
		turn,
		player,
		legalMoves: turn === player ? legalMoves : [],
	} as const
}

function getTurnFromFen(fen: string): Player {
	const activeColor = fen.split(" ")[1]
	switch (activeColor) {
		case "w": {
			return "white"
		}
		case "b": {
			return "black"
		}
		default: {
			throw new Error(`Invalid FEN active color: ${fen}`)
		}
	}
}

function opponent(player: Player): Player {
	return player === "white" ? "black" : "white"
}

function createBoardFromFen(fen: string) {
	const [placement] = fen.split(" ")
	const nextBoard: (Piece | undefined)[] = []

	for (const rank of placement?.split("/") ?? []) {
		for (const char of rank) {
			const emptySquares = Number(char)
			if (Number.isInteger(emptySquares) && emptySquares > 0) {
				for (let square = 0; square < emptySquares; square += 1) {
					nextBoard.push(undefined)
				}
				continue
			}

			if (isPiece(char)) {
				nextBoard.push(char)
				continue
			}

			throw new Error(`Invalid FEN: ${fen}`)
		}
	}

	if (nextBoard.length !== 64) throw new Error(`Invalid FEN: ${fen}`)
	return nextBoard
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

function isPiece(piece: string): piece is Piece {
	return PIECES.includes(piece)
}

if (import.meta.vitest) {
	const { it, expect, vi } = import.meta.vitest
	it.concurrent("returns valid board state for initial fen", () => {
		expect(createBoardFromFen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")).toEqual([
			..."rnbqkbnrpppppppp".split(""),
			...Array.from({ length: 32 }, () => undefined),
			..."PPPPPPPPRNBQKBNR".split(""),
		])
	})

	it.concurrent("ignores illegal moves and applies legal moves", () => {
		const store = createBoardStore({
			player: "white",
			snapshot: {
				fen: "8/8/8/8/8/8/4P3/8 w - - 0 1",
				legalMoves: [{ from: 52, to: 44 }],
			},
		})

		store.movePiece({ from: 52, to: 36 })
		expect(store.getState().board[52]).toBe("P")
		expect(store.getState().board[36]).toBeUndefined()

		store.movePiece({ from: 52, to: 44 })
		expect(store.getState().board[52]).toBeUndefined()
		expect(store.getState().board[44]).toBe("P")
		expect(store.getState().turn).toBe("black")
		expect(store.getState().legalMoves).toEqual([])
	})

	it.concurrent("calls onMove only for legal moves", () => {
		const onMove = vi.fn()
		const store = createBoardStore({
			player: "white",
			onMove,
			snapshot: {
				fen: "8/8/8/8/8/8/4P3/8 w - - 0 1",
				legalMoves: [{ from: 52, to: 44 }],
			},
		})

		store.movePiece({ from: 52, to: 36 })
		store.movePiece({ from: 52, to: 52 })
		expect(onMove).not.toHaveBeenCalled()

		store.movePiece({ from: 52, to: 44 })
		expect(onMove).toHaveBeenCalledExactlyOnceWith({ from: 52, to: 44 })
	})

	it.concurrent("filters legal moves to the current player", () => {
		const legalMoves = [{ from: 52, to: 44 }]
		const store = createBoardStore({
			player: "black",
			snapshot: { fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", legalMoves },
		})

		expect(store.getState().turn).toBe("white")
		expect(store.getState().legalMoves).toEqual([])

		store.setState({ fen: "8/3p4/8/8/8/8/8/8 b - - 0 1", legalMoves })

		expect(store.getState().turn).toBe("black")
		expect(store.getState().legalMoves).toEqual(legalMoves)
	})

	it.concurrent("applies remote moves and filters legal moves by turn", () => {
		const legalMoves = [{ from: 11, to: 19 }]
		const store = createBoardStore({
			player: "black",
			snapshot: { fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", legalMoves: [] },
		})

		store.applyMove({ move: { from: 52, to: 44 }, turn: "black", legalMoves })

		expect(store.getState().board[52]).toBeUndefined()
		expect(store.getState().board[44]).toBe("P")
		expect(store.getState().turn).toBe("black")
		expect(store.getState().legalMoves).toEqual(legalMoves)
	})

	it.concurrent("hides legal moves when it is the opponent's turn", () => {
		const legalMoves = [{ from: 52, to: 44 }]
		const store = createBoardStore({
			player: "black",
			snapshot: { fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", legalMoves: [] },
		})

		store.applyMove({ move: { from: 52, to: 44 }, turn: "white", legalMoves })

		expect(store.getState().board[52]).toBeUndefined()
		expect(store.getState().board[44]).toBe("P")
		expect(store.getState().turn).toBe("white")
		expect(store.getState().legalMoves).toEqual([])
	})
}
