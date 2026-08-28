import { bench, describe, inject } from "vitest"
import { z } from "zod"

import { Perft } from "../build/benchmark/game_server.js"

const perftCasesSchema = z
	.array(z.object({ depth: z.number().int().nonnegative(), fen: z.string().min(1) }))
	.min(1)

const casesJson = inject("PERFT_CASES")
const label = inject("PERFT_LABEL")

if (!casesJson || !label) {
	throw new Error(`Invalid perft benchmark configuration: ${JSON.stringify({ casesJson, label })}`)
}

const cases = perftCasesSchema.parse(JSON.parse(casesJson))
const perfts = cases.map(({ depth, fen }) => ({ depth, perft: new Perft(fen) }))
const nodesPerIteration = perfts.reduce((nodes, { depth, perft }) => nodes + perft.run(depth), 0n)

describe("perft", () => {
	bench(
		`perft/${label}/nodes-${nodesPerIteration}`,
		() => {
			for (const { depth, perft } of perfts) perft.run(depth)
		},
		{ time: 5_000, warmupTime: 3_000 },
	)
})
