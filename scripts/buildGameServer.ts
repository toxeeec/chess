import { execFileSync } from "node:child_process"
import { readFileSync, renameSync, rmSync, writeFileSync } from "node:fs"
import { parseArgs } from "node:util"

type ModeConfiguration = {
	cargoFlags: string[]
	optimize: boolean
	profile: string
}

const modeConfiguration: Record<string, ModeConfiguration | undefined> = {
	dev: {
		cargoFlags: [],
		optimize: false,
		profile: "debug",
	},
	benchmark: {
		cargoFlags: ["--profile", "benchmark-wasm", "--features", "benchmark"],
		optimize: true,
		profile: "benchmark-wasm",
	},
}

const { values } = parseArgs({
	options: { mode: { type: "string" } },
	strict: true,
})
const mode = values.mode
if (!mode) {
	console.error("Build mode is required")
	process.exit(1)
}

const configuration = modeConfiguration[mode]
if (!configuration || !Object.hasOwn(modeConfiguration, mode)) {
	console.error(
		`Build mode must be one of: ${Object.keys(modeConfiguration).join(", ")}; received: ${mode}`,
	)
	process.exit(1)
}

const { cargoFlags, optimize, profile } = configuration
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
if (optimize) optimizeWasm(`${outputDirectory}/game_server_bg.wasm`)
patchWasmImport(`${outputDirectory}/game_server.js`)

function optimizeWasm(file: string) {
	const optimizedFile = `${file}.optimized`
	try {
		execFileSync("wasm-opt", [file, "-O3", "-o", optimizedFile], { stdio: "inherit" })
		renameSync(optimizedFile, file)
	} finally {
		rmSync(optimizedFile, { force: true })
	}
}

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
