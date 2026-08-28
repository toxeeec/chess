declare module "vitest" {
	export interface ProvidedContext {
		GAME_DATASET: string
		GAME_DATASET_ID: string
		PERFT_CASES: string
		PERFT_LABEL: string
		TEST_SCHEMA_SQL: string
	}
}

// oxlint-disable-next-line unicorn/require-module-specifiers
export {}
