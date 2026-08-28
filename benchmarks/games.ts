import { existsSync, mkdirSync, writeFileSync } from "node:fs"
import { dirname, resolve } from "node:path"

import { Chess } from "chess.js"

import type { BenchmarkTarget } from "./report.ts"
import { runBenchmark } from "./report.ts"

const GAME_DATASET_ID = "world-championships-2021-2023-2024"
const GAME_DATASET_PATH = resolve(`.cache/benchmarks/${GAME_DATASET_ID}.json`)
const SOURCES = [
	"https://www.pgnmentor.com/events/WorldChamp2021.pgn",
	"https://www.pgnmentor.com/events/WorldChamp2023.pgn",
	"https://www.pgnmentor.com/events/WorldChamp2024.pgn",
] as const
const EXPECTED_GAMES = 39

export async function runGamesBenchmark(target: BenchmarkTarget) {
	await prepareGameDataset()
	runBenchmark({
		benchmark: "games",
		datasetId: GAME_DATASET_ID,
		datasetPath: GAME_DATASET_PATH,
		target,
	})
}

async function prepareGameDataset() {
	if (existsSync(GAME_DATASET_PATH)) return

	console.error(`Preparing ${GAME_DATASET_ID} benchmark dataset...\n`)
	const games = [] as string[][]
	const pgns = await Promise.all(
		SOURCES.map(async (source) => {
			const response = await fetch(source)
			if (!response.ok) {
				throw new Error(`Failed to download ${source}: ${response.status} ${response.statusText}`)
			}

			return response.text()
		}),
	)

	for (const pgn of pgns) {
		for (const gamePgn of pgn.trim().split(/(?=\[Event )/)) {
			const game = new Chess()
			game.loadPgn(gamePgn)
			games.push(
				game
					.history({ verbose: true })
					.map((move) => `${move.from}${move.to}${move.promotion ?? ""}`),
			)
		}
	}
	if (games.length !== EXPECTED_GAMES) {
		throw new Error(`Expected ${EXPECTED_GAMES} games, received ${games.length}`)
	}

	mkdirSync(dirname(GAME_DATASET_PATH), { recursive: true })
	writeFileSync(GAME_DATASET_PATH, JSON.stringify(games))
}
