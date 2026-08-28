import { bench, describe, inject } from "vitest"

import { GameReplay } from "../build/benchmark/game_server.js"

const gameDatasetId = inject("GAME_DATASET_ID")
const replay = new GameReplay(inject("GAME_DATASET"))
const movesPerReplay = replay.run()
const replaysPerIteration = 50

describe("games", () => {
	bench(
		`games/${gameDatasetId}/moves-${movesPerReplay * replaysPerIteration}`,
		() => {
			for (let index = 0; index < replaysPerIteration; index++) {
				replay.run()
			}
		},
		{ time: 5_000, warmupTime: 3_000 },
	)
})
