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
	plugins: ["typescript", "unicorn", "oxc", "import", "react", "jsx-a11y", "node", "promise"],
	rules: { "react/react-in-jsx-scope": "off" },
})
