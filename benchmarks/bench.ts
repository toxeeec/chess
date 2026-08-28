import { parseArgs } from "node:util"

import { runGamesBenchmark } from "./games.ts"
import { PERFT_SUITE, POSITIONS, runPerftBenchmark, runPerftSuite } from "./perft.ts"

const {
	positionals: [benchmark, ...args],
	values: { target },
} = parseArgs({
	allowPositionals: true,
	options: { target: { type: "string" } },
	strict: true,
})

if (target !== "native" && target !== "wasm") {
	exitWithUsage()
}

if (benchmark === "games" && args.length === 0) {
	await runGamesBenchmark(target)
} else if (benchmark === "perft" && args.length === 1 && args[0] === "suite") {
	runPerftSuite(target)
} else if (benchmark === "perft" && args.length === 2) {
	const [position, depth] = args
	if (!position || depth === undefined) exitWithUsage()
	runPerftBenchmark(target, position, depth)
} else {
	exitWithUsage()
}

function exitWithUsage(): never {
	console.error(`Usage:
  pnpm bench games --target <native|wasm>
  pnpm bench perft suite --target <native|wasm>
  pnpm bench perft <position-or-fen> <depth> --target <native|wasm>

Known positions: ${[...POSITIONS.keys()].join(", ")}
Suite depths: ${[...PERFT_SUITE].map(([position, depth]) => `${position}=${depth}`).join(", ")}`)
	process.exit(1)
}
