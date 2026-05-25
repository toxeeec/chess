import { defineConfig } from "oxfmt"

export default defineConfig({
	ignorePatterns: ["src/routeTree.gen.ts", "pnpm-lock.yaml", "worker-configuration.d.ts"],
	printWidth: 100,
	semi: false,
	sortImports: true,
	sortTailwindcss: {
		functions: ["clsx"],
		stylesheet: "./src/styles.css",
	},
	useTabs: true,
})
