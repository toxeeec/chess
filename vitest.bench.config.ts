import { readFileSync } from "node:fs"

import { cloudflareTest } from "@cloudflare/vitest-plugin"
import { defineConfig } from "vitest/config"
import { unstable_readConfig } from "wrangler"

const gameDatasetPath = process.env.GAME_DATASET_PATH
const wranglerConfig = unstable_readConfig({ config: "wrangler.jsonc" })
if (!wranglerConfig.compatibility_date) {
	throw new Error("Wrangler compatibility_date is required")
}

export default defineConfig({
	plugins: [
		cloudflareTest({
			miniflare: {
				compatibilityDate: wranglerConfig.compatibility_date,
				compatibilityFlags: wranglerConfig.compatibility_flags,
				modulesRules: [{ type: "CompiledWasm", include: ["**/*.wasm?module"] }],
			},
		}),
	],
	test: {
		include: ["game-server/benches/**/*.bench.ts"],
		provide: {
			GAME_DATASET: gameDatasetPath ? readFileSync(gameDatasetPath, "utf8") : "",
			GAME_DATASET_ID: process.env.GAME_DATASET_ID,
			PERFT_CASES: process.env.PERFT_CASES,
			PERFT_LABEL: process.env.PERFT_LABEL,
		},
	},
})
