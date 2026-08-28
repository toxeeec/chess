import { execFileSync } from "node:child_process"
import { readFileSync, writeFileSync } from "node:fs"
import { parseArgs } from "node:util"

type ModeConfiguration = { cargoFlags: string[]; profile: string }

const modeConfiguration: Record<string, ModeConfiguration | undefined> = {
	dev: { cargoFlags: [], profile: "debug" },
	benchmark: {
		cargoFlags: ["--profile", "benchmark", "--features", "benchmark"],
		profile: "benchmark",
	},
}

const { values } = parseArgs({
	options: { mode: { type: "string" } },
	strict: true,
})
const mode = values.mode
if (mode === undefined) {
	console.error("Build mode is required")
	process.exit(1)
}

const configuration = modeConfiguration[mode]
if (configuration === undefined || !Object.hasOwn(modeConfiguration, mode)) {
	console.error(
		`Build mode must be one of: ${Object.keys(modeConfiguration).join(", ")}; received: ${mode}`,
	)
	process.exit(1)
}

const { cargoFlags, profile } = configuration
const outputDirectory = `game-server/build/${mode}`

execFileSync(
	"cargo",
	[
		"build",
		...cargoFlags,
		"--manifest-path",
		"game-server/Cargo.toml",
		"--target",
		"wasm32-unknown-unknown",
		"-Zbuild-std=std,panic_unwind",
	],
	{
		env: { ...process.env, RUSTFLAGS: "-Cpanic=unwind" },
		stdio: "inherit",
	},
)
execFileSync(
	"wasm-bindgen",
	[
		`game-server/target/wasm32-unknown-unknown/${profile}/game_server.wasm`,
		"--target",
		"module",
		"--typescript",
		"--out-dir",
		outputDirectory,
		"--experimental-reset-state-function",
	],
	{ stdio: "inherit" },
)
patchWasmImport(`${outputDirectory}/game_server.js`)

function patchWasmImport(file: string) {
	const importSource = 'import source wasmModule from "./game_server_bg.wasm"'
	const importModule = 'import wasmModule from "./game_server_bg.wasm?module"'
	const source = readFileSync(file, "utf8")
	const patched = source.replace(importSource, importModule)

	if (patched === source && !source.includes(importModule)) {
		throw new Error("Expected wasm-bindgen module import was not found")
	}

	writeFileSync(file, patched)
}
