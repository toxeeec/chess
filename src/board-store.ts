import { createContext, use, useSyncExternalStore } from "react"

export const PIECES = ["r", "n", "b", "q", "k", "p", "R", "N", "B", "Q", "K", "P"] as const
export type Piece = (typeof PIECES)[number]

export type BoardStore = ReturnType<typeof createBoardStore>
type BoardState = ReturnType<BoardStore["getState"]>
type Snapshot = Parameters<typeof createBoardState>[0]

export function createBoardStore(snapshot: Snapshot) {
	let state = createBoardState(snapshot)
	const listeners = new Set<() => void>()
	const notify = () => {
		for (const listener of listeners) {
			listener()
		}
	}

	return {
		getState: () => state,
		setState: (snapshot: Snapshot) => {
			state = createBoardState(snapshot)
			notify()
		},
		setDraggedPieceSquare: (draggedPieceSquare: number | null) => {
			if (state.draggedPieceSquare === draggedPieceSquare) return

			state = { ...state, draggedPieceSquare }
			notify()
		},
		movePiece: (sourceSquare: number, targetSquare: number) => {
			const movingPiece = state.board[sourceSquare]
			if (!movingPiece || sourceSquare === targetSquare) return
			if (!state.legalMoves.some(({ from, to }) => from === sourceSquare && to === targetSquare))
				return

			const board = [...state.board]
			board[sourceSquare] = undefined
			board[targetSquare] = movingPiece

			state = { board, legalMoves: [], draggedPieceSquare: null }
			notify()
		},
		subscribe: (listener: () => void) => {
			listeners.add(listener)
			return () => listeners.delete(listener)
		},
	}
}

export const BoardStoreContext = createContext<BoardStore | null>(null)

function createBoardState({ fen, moves }: { fen: string; moves: readonly string[] }) {
	return {
		board: createBoardFromFen(fen),
		draggedPieceSquare: null as number | null,
		legalMoves: moves.map((move) => {
			const fromFile = move[0]!.charCodeAt(0) - "a".charCodeAt(0)
			const fromRank = 8 - Number(move[1])
			const toFile = move[2]!.charCodeAt(0) - "a".charCodeAt(0)
			const toRank = 8 - Number(move[3])
			return { from: fromRank * 8 + fromFile, to: toRank * 8 + toFile }
		}),
	}
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
	const { it, expect } = import.meta.vitest
	it("returns valid board state for initial fen", () => {
		expect(createBoardFromFen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")).toEqual([
			..."rnbqkbnrpppppppp".split(""),
			...Array.from({ length: 32 }, () => undefined),
			..."PPPPPPPPRNBQKBNR".split(""),
		])
	})

	it("ignores illegal moves and applies legal moves", () => {
		const store = createBoardStore({ fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", moves: ["e2e3"] })

		store.movePiece(52, 36)
		expect(store.getState().board[52]).toBe("P")
		expect(store.getState().board[36]).toBeUndefined()

		store.movePiece(52, 44)
		expect(store.getState().board[52]).toBeUndefined()
		expect(store.getState().board[44]).toBe("P")
		expect(store.getState().legalMoves).toEqual([])
	})
}
