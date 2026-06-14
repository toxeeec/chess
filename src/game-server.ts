import { DurableObject } from "cloudflare:workers"

import { GameServer as WasmGameServer } from "../game-server/build/game_server.js"

export class GameServer extends DurableObject {
	readonly #wasmGameServer: WasmGameServer

	constructor(ctx: DurableObjectState, env: Env) {
		super(ctx, env)
		this.#wasmGameServer = new WasmGameServer(ctx, env)
	}

	init({
		joinTimeoutMs,
		firstMoveTimeoutMs,
		disconnectTimeoutMs,
	}: {
		joinTimeoutMs: number
		firstMoveTimeoutMs: number
		disconnectTimeoutMs: number
	}) {
		return this.#wasmGameServer.init(joinTimeoutMs, firstMoveTimeoutMs, disconnectTimeoutMs)
	}

	snapshot() {
		const { revision, fen, status, legalMoves } = this.#wasmGameServer.snapshot()
		return { revision, fen, status, legalMoves }
	}

	fetch(request: Request) {
		return this.#wasmGameServer.fetch(request)
	}

	webSocketMessage(ws: WebSocket, message: string) {
		return this.#wasmGameServer.webSocketMessage(ws, message)
	}

	webSocketClose(ws: WebSocket, code: number, reason: string, was_clean: boolean) {
		return this.#wasmGameServer.webSocketClose(ws, code, reason, was_clean)
	}

	alarm() {
		return this.#wasmGameServer.alarm()
	}
}
