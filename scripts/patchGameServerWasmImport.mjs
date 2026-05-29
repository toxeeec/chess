import { readFileSync, writeFileSync } from "node:fs"

const file = new URL("../game-server/build/game_server.js", import.meta.url)
const importSource = 'import source wasmModule from "./game_server_bg.wasm"'
const importModule = 'import wasmModule from "./game_server_bg.wasm?module"'

const source = readFileSync(file, "utf8")
const patched = source.replace(importSource, importModule)

if (patched === source && !source.includes(importModule)) {
	throw new Error("Expected wasm-bindgen module import was not found")
}

writeFileSync(file, patched)
