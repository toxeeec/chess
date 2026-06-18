import { defineConfig } from "oxlint"

export default defineConfig({
	categories: {
		correctness: "error",
		perf: "error",
		suspicious: "error",
	},
	env: {
		browser: true,
		builtin: true,
	},
	options: {
		typeAware: true,
		typeCheck: true,
	},
	plugins: [
		"typescript",
		"unicorn",
		"oxc",
		"import",
		"react",
		"jsx-a11y",
		"node",
		"promise",
		"vitest",
	],
	rules: {
		"no-shadow": "off",
		"react/react-compiler": "error",
		"react/react-in-jsx-scope": "off",
		"typescript/no-explicit-any": "error",
	},
	overrides: [
		{
			files: ["**/*.test.ts", "**/*.test.tsx", "**/*.spec.ts", "**/*.spec.tsx"],
			rules: {
				"typescript/no-explicit-any": "off",
				"typescript/no-unsafe-type-assertion": "off",
			},
		},
	],
})
