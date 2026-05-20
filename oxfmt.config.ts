import { defineConfig } from "oxfmt"

export default defineConfig({
	ignorePatterns: ["src/routeTree.gen.ts", "pnpm-lock.yaml"],
	printWidth: 100,
	semi: false,
	sortImports: true,
	sortTailwindcss: true,
	useTabs: true,
})
