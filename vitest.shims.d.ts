declare module "vitest" {
	export interface ProvidedContext {
		TEST_SCHEMA_SQL: string
	}
}

// oxlint-disable-next-line unicorn/require-module-specifiers
export {}
