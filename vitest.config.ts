import { execFileSync } from "node:child_process"

import { cloudflareTest } from "@cloudflare/vitest-pool-workers"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import { defineConfig } from "vitest/config"

const testSchemaSql = execFileSync("pnpm", ["exec", "drizzle-kit", "export"], {
	encoding: "utf8",
}).replaceAll("\n", " ")

export default defineConfig({
	plugins: [
		tanstackStart(),
		cloudflareTest({
			main: "./src/server.ts",
			miniflare: {
				compatibilityDate: "2025-09-02",
				compatibilityFlags: ["nodejs_compat"],
				d1Databases: { DB: "chess-test" },
				durableObjects: { GAME_SERVER: { className: "GameServer", useSQLite: true } },
				modulesRules: [{ type: "CompiledWasm", include: ["**/*.wasm?module"] }],
			},
		}),
	],
	resolve: { tsconfigPaths: true },
	test: {
		includeSource: ["src/**/*.{ts,tsx}"],
		provide: { TEST_SCHEMA_SQL: testSchemaSql },
	},
})
