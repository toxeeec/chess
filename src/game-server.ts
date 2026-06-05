import { DurableObject } from "cloudflare:workers"

import { GameServer as WasmGameServer } from "../game-server/build/game_server.js"

export class GameServer extends DurableObject {
	readonly #wasmGameServer: WasmGameServer

	constructor(ctx: DurableObjectState, env: Env) {
		super(ctx, env)
		this.#wasmGameServer = new WasmGameServer(ctx, env)
	}

	snapshot() {
		const { revision, fen, legalMoves } = this.#wasmGameServer.snapshot()
		return { revision, fen, legalMoves }
	}

	async clear() {
		if (import.meta.env.MODE !== "test") {
			throw new Error("GameServer.clear can only be called in tests")
		}

		await Promise.all([this.ctx.storage.deleteAlarm(), this.ctx.storage.deleteAll()])
	}

	fetch(request: Request) {
		return this.#wasmGameServer.fetch(request)
	}

	webSocketMessage(ws: WebSocket, message: string) {
		return this.#wasmGameServer.webSocketMessage(ws, message)
	}

	webSocketClose() {}
}
