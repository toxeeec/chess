import { execFileSync } from "node:child_process"
import { resolve } from "node:path"

import { cloudflare } from "@cloudflare/vite-plugin"
import babel from "@rolldown/plugin-babel"
import tailwindcss from "@tailwindcss/vite"
import { devtools } from "@tanstack/devtools-vite"
import { tanstackStart } from "@tanstack/react-start/plugin/vite"
import viteReact, { reactCompilerPreset } from "@vitejs/plugin-react"
import { defineConfig, type Plugin } from "vite"

function buildGameServer() {
	execFileSync("pnpm", ["run", "build:game-server"], {
		stdio: "inherit",
	})
}

function gameServer(): Plugin {
	let didInitialBuild = false
	const root = resolve(import.meta.dirname, "game-server")
	const src = resolve(root, "src")
	const cargoToml = resolve(root, "Cargo.toml")
	const cargoLock = resolve(root, "Cargo.lock")
	return {
		name: "game-server",
		buildStart() {
			if (didInitialBuild) return
			didInitialBuild = true
			buildGameServer()
		},
		configureServer(server) {
			server.watcher.add([src, cargoToml, cargoLock])
		},
		async handleHotUpdate({ file, server }) {
			if (file.startsWith(src) || file === cargoToml || file === cargoLock) {
				buildGameServer()
				await server.restart()
			}
		},
	}
}

export default defineConfig({
	define: {
		"import.meta.vitest": "undefined",
	},
	plugins: [
		devtools(),
		gameServer(),
		cloudflare({ viteEnvironment: { name: "ssr" } }),
		tailwindcss(),
		tanstackStart(),
		viteReact(),
		babel({ presets: [reactCompilerPreset()] }),
	],
	resolve: { tsconfigPaths: true },
})
