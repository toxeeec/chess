import { createHash } from "node:crypto"

import type { BenchmarkTarget } from "./report.ts"
import { runBenchmark } from "./report.ts"

export const POSITIONS = new Map([
	["initial", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"],
	["kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"],
	["endgame-en-passant", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"],
	["promotions-castling", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"],
	["tactical-promotions", "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 0 1"],
	[
		"pins-discovered-attacks",
		"r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 1",
	],
])

export const PERFT_SUITE = new Map([
	["initial", 5],
	["kiwipete", 4],
	["endgame-en-passant", 6],
	["promotions-castling", 5],
	["tactical-promotions", 4],
	["pins-discovered-attacks", 4],
])

export function runPerftSuite(target: BenchmarkTarget) {
	const cases = [...PERFT_SUITE].map(([position, depth]) => {
		const fen = POSITIONS.get(position)!
		return { depth, fen }
	})
	runBenchmark({ benchmark: "perft", cases, label: "suite", target })
}

export function runPerftBenchmark(
	target: BenchmarkTarget,
	positionArgument: string,
	depthArgument: string,
) {
	const depth = Number(depthArgument)
	if (!Number.isSafeInteger(depth) || depth < 0) {
		console.error(`Invalid depth: ${depthArgument}`)
		process.exit(1)
	}

	const knownFen = POSITIONS.get(positionArgument)
	const fen = knownFen ?? positionArgument
	const label = knownFen
		? positionArgument
		: `fen-${createHash("sha256").update(fen).digest("hex").slice(0, 8)}`

	runBenchmark({
		benchmark: "perft",
		cases: [{ depth, fen }],
		label: `${label}/depth-${depth}`,
		target,
	})
}
