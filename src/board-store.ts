import { createContext, use, useSyncExternalStore } from "react"

export const PIECES = ["r", "n", "b", "q", "k", "p", "R", "N", "B", "Q", "K", "P"] as const
export type Piece = (typeof PIECES)[number]

type Board = (Piece | undefined)[]
type BoardState = { board: Board }
export type BoardStore = ReturnType<typeof createBoardStore>

export function createBoardStore(fen: string) {
	let state: BoardState = { board: createBoardFromFen(fen) }
	const listeners = new Set<() => void>()
	const notify = () => {
		for (const listener of listeners) {
			listener()
		}
	}

	return {
		getState: () => state,
		setState: (nextState: BoardState) => {
			state = nextState
			notify()
		},
		subscribe: (listener: () => void) => {
			listeners.add(listener)
			return () => listeners.delete(listener)
		},
		movePiece: (sourceSquare: number, targetSquare: number) => {
			const movingPiece = state.board[sourceSquare]
			if (!movingPiece || sourceSquare === targetSquare) return

			const board = [...state.board]
			board[sourceSquare] = undefined
			board[targetSquare] = movingPiece

			state = { board }
			notify()
		},
	}
}

export const BoardStoreContext = createContext<BoardStore | null>(null)

export function createBoardFromFen(fen: string) {
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
	const { it, expect } = import.meta.vitest
	it("returns valid board state for initial fen", () => {
		expect(createBoardFromFen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")).toEqual([
			..."rnbqkbnrpppppppp".split(""),
			...Array.from({ length: 32 }, () => undefined),
			..."PPPPPPPPRNBQKBNR".split(""),
		])
	})
}
