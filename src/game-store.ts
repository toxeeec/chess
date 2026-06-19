import { createContext, use, useRef, useSyncExternalStore } from "react"

import type { Player } from "./room"
import type { Clock, Move } from "./use-live-room"

const PIECES = ["r", "n", "b", "q", "k", "p", "R", "N", "B", "Q", "K", "P"] as const
export type Piece = (typeof PIECES)[number]

type GameStore = ReturnType<typeof createGameStore>
type GameState = ReturnType<GameStore["getState"]>
type Snapshot = { fen: string; legalMoves: readonly Move[]; clock: Clock }

export function createGameStore({
	snapshot: { fen, legalMoves, clock },
	player,
	onMove,
}: {
	snapshot: Snapshot
	player: Player
	onMove?: (move: Move) => void
}) {
	const turn = getTurnFromFen(fen)
	let state = {
		board: createBoardFromFen(fen),
		turn,
		player,
		clock: { ...clock, receivedAtMs: Date.now() },
		legalMoves: turn === player ? legalMoves : [],
	}

	const listeners = new Set<() => void>()
	const notify = () => {
		for (const listener of listeners) {
			listener()
		}
	}

	return {
		getState: () => state,
		setState: (snapshot: Partial<Snapshot>) => {
			const turn = snapshot.fen ? getTurnFromFen(snapshot.fen) : state.turn
			state = {
				...state,
				turn,
				...(snapshot.fen && {
					board: createBoardFromFen(snapshot.fen),
				}),
				...(snapshot.clock && {
					clock: { ...snapshot.clock, receivedAtMs: Date.now() },
				}),
				...(snapshot.legalMoves && {
					legalMoves: turn === player ? snapshot.legalMoves : [],
				}),
			}
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
				clock: switchClock(state.clock, state.player, state.turn),
				legalMoves: [],
			}

			onMove?.(move)
			notify()
		},
		applyMove: ({
			move,
			legalMoves,
			turn,
			clock,
		}: {
			move: Move
			legalMoves: readonly Move[]
			turn: Player
			clock: Clock
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
				clock: { ...clock, receivedAtMs: Date.now() },
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

export const GameStoreContext = createContext<GameStore | null>(null)

export function useGameStore<T>(selector: (state: GameState) => T) {
	const store = use(GameStoreContext)
	if (!store) throw new Error("useGameStore must be used within GameStoreContext")

	return useSyncExternalStore(
		store.subscribe,
		() => selector(store.getState()),
		() => selector(store.getState()),
	)
}

export function useShallow<State, Selected extends readonly unknown[]>(
	selector: (state: State) => Selected,
) {
	const previous = useRef<Selected>(undefined)
	const hasPrevious = useRef(false)

	return (state: State) => {
		const next = selector(state)

		if (hasPrevious.current && shallow(previous.current!, next)) {
			return previous.current!
		}

		previous.current = next
		hasPrevious.current = true
		return next
	}
}

function switchClock(clock: GameState["clock"], player: Player, turn: Player) {
	const now = Date.now()
	const active = clock.running && turn === player
	const key = player === "white" ? "whiteRemainingMs" : "blackRemainingMs"
	const currentRemainingMs = clock[key]

	const remainingMs = active
		? Math.max(0, currentRemainingMs - (now - clock.receivedAtMs))
		: currentRemainingMs

	return {
		...clock,
		[key]: remainingMs,
		receivedAtMs: now,
		running: true,
	}
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

function isPiece(piece: string): piece is Piece {
	return PIECES.includes(piece)
}

function shallow<T extends readonly unknown[]>(a: T, b: T) {
	if (Object.is(a, b)) return true
	if (a.length !== b.length) return false

	for (let i = 0; i < a.length; ++i) {
		if (!Object.is(a[i], b[i])) return false
	}
	return true
}

if (import.meta.vitest) {
	const { it, expect, vi } = import.meta.vitest
	const TEST_CLOCK = {
		whiteRemainingMs: 300_000,
		blackRemainingMs: 300_000,
		running: false,
	} as const satisfies Clock

	it.concurrent("returns valid board state for initial fen", () => {
		expect(createBoardFromFen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")).toEqual([
			..."rnbqkbnrpppppppp".split(""),
			...Array.from({ length: 32 }, () => undefined),
			..."PPPPPPPPRNBQKBNR".split(""),
		])
	})

	it.concurrent("ignores illegal moves and applies legal moves", () => {
		const store = createGameStore({
			player: "white",
			snapshot: {
				fen: "8/8/8/8/8/8/4P3/8 w - - 0 1",
				clock: TEST_CLOCK,
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
		const store = createGameStore({
			player: "white",
			onMove,
			snapshot: {
				fen: "8/8/8/8/8/8/4P3/8 w - - 0 1",
				clock: TEST_CLOCK,
				legalMoves: [{ from: 52, to: 44 }],
			},
		})

		store.movePiece({ from: 52, to: 36 })
		store.movePiece({ from: 52, to: 52 })
		expect(onMove).not.toHaveBeenCalled()

		store.movePiece({ from: 52, to: 44 })
		expect(onMove).toHaveBeenCalledExactlyOnceWith({ from: 52, to: 44 })
	})

	it.concurrent("does not move pieces without a matching legal move", () => {
		const onMove = vi.fn()
		const store = createGameStore({
			player: "white",
			onMove,
			snapshot: {
				fen: "8/8/8/8/8/8/4P3/8 w - - 0 1",
				clock: TEST_CLOCK,
				legalMoves: [],
			},
		})

		store.movePiece({ from: 52, to: 44 })

		expect(store.getState().board[52]).toBe("P")
		expect(store.getState().board[44]).toBeUndefined()
		expect(onMove).not.toHaveBeenCalled()
	})

	it.concurrent("filters legal moves to the current player", () => {
		const legalMoves = [{ from: 52, to: 44 }]
		const store = createGameStore({
			player: "black",
			snapshot: { fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", clock: TEST_CLOCK, legalMoves },
		})

		expect(store.getState().turn).toBe("white")
		expect(store.getState().legalMoves).toEqual([])

		store.setState({ fen: "8/3p4/8/8/8/8/8/8 b - - 0 1", clock: TEST_CLOCK, legalMoves })

		expect(store.getState().turn).toBe("black")
		expect(store.getState().legalMoves).toEqual(legalMoves)
	})

	it.concurrent("updates the clock from snapshots", () => {
		const nextClock = {
			whiteRemainingMs: 299_000,
			blackRemainingMs: 300_000,
			running: true,
		} as const satisfies Clock
		const store = createGameStore({
			player: "white",
			snapshot: { fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", clock: TEST_CLOCK, legalMoves: [] },
		})

		store.setState({ clock: nextClock })

		expect(store.getState().clock).toMatchObject(nextClock)
	})

	it.concurrent("applies remote moves and filters legal moves by turn", () => {
		const legalMoves = [{ from: 11, to: 19 }]
		const store = createGameStore({
			player: "black",
			snapshot: { fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", clock: TEST_CLOCK, legalMoves: [] },
		})

		store.applyMove({ move: { from: 52, to: 44 }, turn: "black", clock: TEST_CLOCK, legalMoves })

		expect(store.getState().board[52]).toBeUndefined()
		expect(store.getState().board[44]).toBe("P")
		expect(store.getState().turn).toBe("black")
		expect(store.getState().legalMoves).toEqual(legalMoves)
	})

	it.concurrent("hides legal moves when it is the opponent's turn", () => {
		const legalMoves = [{ from: 52, to: 44 }]
		const store = createGameStore({
			player: "black",
			snapshot: { fen: "8/8/8/8/8/8/4P3/8 w - - 0 1", clock: TEST_CLOCK, legalMoves: [] },
		})

		store.applyMove({ move: { from: 52, to: 44 }, turn: "white", clock: TEST_CLOCK, legalMoves })

		expect(store.getState().board[52]).toBeUndefined()
		expect(store.getState().board[44]).toBe("P")
		expect(store.getState().turn).toBe("white")
		expect(store.getState().legalMoves).toEqual([])
	})
}
