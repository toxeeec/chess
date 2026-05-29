import { DurableObject } from "cloudflare:workers"

import { GameServer as WasmGameServer } from "../game-server/build/game_server.js"

export class GameServer extends DurableObject {
	readonly #wasmGameServer: WasmGameServer

	constructor(ctx: DurableObjectState, env: Env) {
		super(ctx, env)
		this.#wasmGameServer = new WasmGameServer(ctx, env)
	}

	fen() {
		return this.#wasmGameServer.fen()
	}
}
