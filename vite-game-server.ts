import { execFileSync } from "node:child_process"
import { existsSync, readdirSync, statSync } from "node:fs"
import { resolve } from "node:path"

import type { Plugin } from "vite"

function buildGameServer() {
	execFileSync("pnpm", ["run", "build:game-server"], {
		stdio: "inherit",
	})
}

function latestMtime(path: string): number {
	const stat = statSync(path)
	if (!stat.isDirectory()) return stat.mtimeMs

	return Math.max(
		stat.mtimeMs,
		...readdirSync(path).map((entry) => latestMtime(resolve(path, entry))),
	)
}

export function gameServer(): Plugin {
	let isServe = false
	let buildTimer: ReturnType<typeof setTimeout> | undefined
	const root = resolve(import.meta.dirname, "game-server")
	const src = resolve(root, "src")
	const cargoToml = resolve(root, "Cargo.toml")
	const cargoLock = resolve(root, "Cargo.lock")
	const patchScript = resolve(import.meta.dirname, "scripts/patchGameServerWasmImport.mjs")
	const outputJs = resolve(root, "build/game_server.js")
	const outputWasm = resolve(root, "build/game_server_bg.wasm")
	const inputs = [src, cargoToml, cargoLock, patchScript]
	const buildIfStale = () => {
		if (!existsSync(outputJs) || !existsSync(outputWasm)) {
			buildGameServer()
			return
		}

		const latestInput = Math.max(...inputs.map(latestMtime))
		const oldestOutput = Math.min(latestMtime(outputJs), latestMtime(outputWasm))

		if (oldestOutput >= latestInput) return
		buildGameServer()
	}
	return {
		name: "game-server",
		configResolved(config) {
			isServe = config.command === "serve"
		},
		buildStart() {
			for (const input of inputs) this.addWatchFile(input)
			if (!isServe) buildIfStale()
		},
		configureServer(server) {
			buildIfStale()
			server.watcher.add(inputs)

			const handleGameServerChange = (file: string) => {
				const changedFile = resolve(file)
				if (!changedFile.startsWith(src) && !inputs.includes(changedFile)) return
				clearTimeout(buildTimer)
				buildTimer = setTimeout(() => {
					buildTimer = undefined
					try {
						buildGameServer()
					} catch (error) {
						const buildError = error instanceof Error ? error : new Error(String(error))
						server.config.logger.error(buildError.message, { error: buildError })
					}
				}, 100)
			}

			server.watcher.on("add", handleGameServerChange)
			server.watcher.on("change", handleGameServerChange)
			server.watcher.on("unlink", handleGameServerChange)
		},
	}
}
