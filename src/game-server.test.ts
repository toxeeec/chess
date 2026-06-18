import { runDurableObjectAlarm } from "cloudflare:test"
import { env } from "cloudflare:workers"
import { describe, expect, it, type DeeplyAllowMatchers } from "vitest"

import type { Player, RoomId } from "./room"
import { generateRoomId } from "./room.server"
import { TEST_GAME_CONFIG } from "./test-utils"

function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms))
}

function clockMatcher(matcher?: DeeplyAllowMatchers<any>) {
	return expect.objectContaining({
		whiteRemainingMs: expect.any(Number),
		blackRemainingMs: expect.any(Number),
		running: expect.any(Boolean),
		...matcher,
	})
}

function snapshotMessageMatcher(dataMatcher?: DeeplyAllowMatchers<any>) {
	const { clock, ...dataMatcherRest } = dataMatcher ?? {}

	return {
		type: "snapshot",
		data: expect.objectContaining({
			clock: clockMatcher(clock),
			fen: expect.any(String),
			revision: expect.any(Number),
			status: expect.any(String),
			legalMoves: expect.toBeOneOf([expect.any(String), null]),
			...dataMatcherRest,
		}),
	}
}

function statusMessageMatcher(dataMatcher?: DeeplyAllowMatchers<any>) {
	const { clock, ...dataMatcherRest } = dataMatcher ?? {}

	return {
		type: "status",
		data: expect.objectContaining({
			status: expect.any(String),
			clock: clockMatcher(clock),
			legalMoves: expect.toBeOneOf([expect.any(String), null]),
			...dataMatcherRest,
		}),
	}
}

function createWebSocketMessageReader(webSocket: WebSocket) {
	const messages: unknown[] = []
	const pendingReads: ((message: unknown) => void)[] = []

	webSocket.addEventListener("message", (event) => {
		expect(event.data).toBeTypeOf("string")
		const message = JSON.parse(event.data)
		const pendingRead = pendingReads.shift()

		if (pendingRead) {
			pendingRead(message)
		} else {
			messages.push(message)
		}
	})

	return () =>
		new Promise<unknown>((resolve, reject) => {
			if (messages.length > 0) {
				resolve(messages.shift())
				return
			}

			const resolveMessage = (message: unknown) => {
				clearTimeout(timeout)
				resolve(message)
			}
			const timeout = setTimeout(() => {
				const index = pendingReads.indexOf(resolveMessage)
				if (index >= 0) pendingReads.splice(index, 1)
				reject(new Error("Timed out waiting for message"))
			}, 100)

			pendingReads.push(resolveMessage)
		})
}

async function acceptWebSocket(roomId: RoomId, player: Player) {
	const response = await env.GAME_SERVER.getByName(roomId).fetch(
		new Request("https://chess.localhost", {
			headers: {
				Upgrade: "websocket",
				"Player-Color": player,
			},
		}),
	)

	expect(response.status).toBe(101)
	expect(response.webSocket).toBeInstanceOf(WebSocket)

	const webSocket = response.webSocket!
	const readMessage = createWebSocketMessageReader(webSocket)
	webSocket.accept()
	return {
		readMessage,
		webSocket,
		[Symbol.dispose]() {
			webSocket.close()
		},
	}
}

async function connectPlayers(roomId: RoomId) {
	const [white, black] = await Promise.all([
		acceptWebSocket(roomId, "white"),
		acceptWebSocket(roomId, "black"),
	])
	await Promise.all([
		expect(white.readMessage()).resolves.toEqual(snapshotMessageMatcher()),
		expect(black.readMessage()).resolves.toEqual(snapshotMessageMatcher()),
	])

	const expected = statusMessageMatcher({
		status: "active",
		clock: { running: false },
		legalMoves: expect.any(String),
	})
	await Promise.all([
		expect(white.readMessage()).resolves.toEqual(expected),
		expect(black.readMessage()).resolves.toEqual(expected),
	])

	return {
		white,
		black,
		[Symbol.dispose]() {
			white[Symbol.dispose]()
			black[Symbol.dispose]()
		},
	}
}

describe("GameServer", () => {
	it.concurrent("join timeout broadcasts expired status", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			joinTimeoutMs: 1,
		})
		using whiteConnection = await acceptWebSocket(roomId, "white")
		await whiteConnection.readMessage()

		await sleep(10)
		await runDurableObjectAlarm(stub)

		await expect(whiteConnection.readMessage()).resolves.toEqual(
			statusMessageMatcher({
				status: "expired",
				clock: { running: false },
				legalMoves: "",
			}),
		)
	})

	it.concurrent("first move timeout broadcasts expired status", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			firstMoveTimeoutMs: 1,
		})
		using connections = await connectPlayers(roomId)

		await sleep(10)
		await runDurableObjectAlarm(stub)

		const expectedMessage = statusMessageMatcher({
			status: "expired",
			clock: { running: false },
			legalMoves: "",
		})
		await Promise.all([
			expect(connections.white.readMessage()).resolves.toEqual(expectedMessage),
			expect(connections.black.readMessage()).resolves.toEqual(expectedMessage),
		])
	})

	it.concurrent("chess clock timeout broadcasts ended status after first move", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			firstMoveTimeoutMs: 1_000,
			timeControlMs: 1,
		})
		using connections = await connectPlayers(roomId)
		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))
		await Promise.all([connections.white.readMessage(), connections.black.readMessage()])

		await sleep(10)
		await runDurableObjectAlarm(stub)

		const expected = statusMessageMatcher({
			status: "ended",
			clock: { blackRemainingMs: 0, running: false },
			legalMoves: "",
		})
		await Promise.all([
			expect(connections.white.readMessage()).resolves.toEqual(expected),
			expect(connections.black.readMessage()).resolves.toEqual(expected),
		])

		using newConnection = await acceptWebSocket(roomId, "white")
		await expect(newConnection.readMessage()).resolves.toEqual(
			snapshotMessageMatcher({
				revision: 1,
				fen: "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b - - 0 1",
				status: "ended",
				clock: { blackRemainingMs: 0, running: false },
				legalMoves: "",
			}),
		)
	})

	it.concurrent("rejects moves after chess clock timeout ends the game", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			firstMoveTimeoutMs: 1_000,
			timeControlMs: 1,
		})
		using connections = await connectPlayers(roomId)
		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))
		await Promise.all([connections.white.readMessage(), connections.black.readMessage()])

		await sleep(10)
		await runDurableObjectAlarm(stub)

		const expected = statusMessageMatcher({
			status: "ended",
			clock: { blackRemainingMs: 0, running: false },
			legalMoves: "",
		})
		await Promise.all([
			expect(connections.white.readMessage()).resolves.toEqual(expected),
			expect(connections.black.readMessage()).resolves.toEqual(expected),
		])

		connections.black.webSocket.send(JSON.stringify({ type: "move", data: "a7a6" }))

		await expect(connections.black.readMessage()).resolves.toEqual({
			type: "error",
			data: "game-not-active",
		})
	})

	it.concurrent("disconnect timeout broadcasts expired status if no moves were made", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			disconnectTimeoutMs: 1,
		})
		using connections = await connectPlayers(roomId)

		connections.white.webSocket.close()
		await sleep(10)
		await runDurableObjectAlarm(stub)

		await expect(connections.black.readMessage()).resolves.toEqual(
			statusMessageMatcher({
				status: "expired",
				clock: { running: false },
				legalMoves: "",
			}),
		)
	})

	it.concurrent("disconnect timeout broadcasts ended status after a move", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			disconnectTimeoutMs: 1,
		})
		using connections = await connectPlayers(roomId)
		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))
		await Promise.all([connections.white.readMessage(), connections.black.readMessage()])

		connections.white.webSocket.close()
		await sleep(10)
		await runDurableObjectAlarm(stub)

		await expect(connections.black.readMessage()).resolves.toEqual(
			statusMessageMatcher({
				status: "ended",
				clock: { running: false },
				legalMoves: "",
			}),
		)
	})

	it.concurrent("returns forbidden when the player header is missing", async () => {
		const roomId = generateRoomId()
		const response = await env.GAME_SERVER.getByName(roomId).fetch(
			new Request("https://chess.localhost", {
				headers: { Upgrade: "websocket" },
			}),
		)

		expect(response.status).toBe(403)
	})

	it.concurrent("returns forbidden when the player header is invalid", async () => {
		const roomId = generateRoomId()
		const response = await env.GAME_SERVER.getByName(roomId).fetch(
			new Request("https://chess.localhost", {
				headers: { Upgrade: "websocket", "Player-Color": "red" },
			}),
		)

		expect(response.status).toBe(403)
	})

	it.concurrent("sends an initial snapshot to connected sockets", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")

		await expect(connection.readMessage()).resolves.toEqual(
			snapshotMessageMatcher({
				revision: 0,
				fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
				status: "waiting",
				legalMoves: "",
			}),
		)
	})

	it.concurrent("broadcasts moves to connected room sockets", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))

		const expected = {
			type: "move",
			data: expect.objectContaining({
				revision: 1,
				move: "e2e3",
				turn: "black",
				clock: expect.objectContaining({
					whiteRemainingMs: TEST_GAME_CONFIG.timeControlMs,
					blackRemainingMs: TEST_GAME_CONFIG.timeControlMs,
					running: true,
				}),
				legalMoves: expect.any(String),
			}),
		}

		await Promise.all([
			expect(connections.white.readMessage()).resolves.toEqual(expected),
			expect(connections.black.readMessage()).resolves.toEqual(expected),
		])
	})

	it.concurrent("sends latest game snapshot to new websocket connections", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))
		await Promise.all([connections.white.readMessage(), connections.black.readMessage()])

		using newConnection = await acceptWebSocket(roomId, "black")

		await expect(newConnection.readMessage()).resolves.toEqual(
			snapshotMessageMatcher({
				revision: 1,
				fen: "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b - - 0 1",
				status: "active",
				clock: { running: true },
				legalMoves: expect.any(String),
			}),
		)
	})

	it.concurrent("sends expired snapshots to new connections after join timeout", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			joinTimeoutMs: 1,
		})
		using whiteConnection = await acceptWebSocket(roomId, "white")
		await whiteConnection.readMessage()

		await sleep(10)
		await runDurableObjectAlarm(stub)

		using blackConnection = await acceptWebSocket(roomId, "black")

		await expect(blackConnection.readMessage()).resolves.toEqual(
			snapshotMessageMatcher({
				revision: 0,
				fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
				status: "expired",
				clock: { running: false },
				legalMoves: "",
			}),
		)
	})

	it.concurrent("sends expired snapshots to new connections after first move timeout", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			firstMoveTimeoutMs: 1,
		})
		using _ = await connectPlayers(roomId)

		await sleep(10)
		await runDurableObjectAlarm(stub)

		using newConnection = await acceptWebSocket(roomId, "white")
		await expect(newConnection.readMessage()).resolves.toEqual(
			snapshotMessageMatcher({
				revision: 0,
				fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
				status: "expired",
				clock: { running: false },
				legalMoves: "",
			}),
		)
	})

	it.concurrent("rejects moves before both players join", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")
		await connection.readMessage()

		connection.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))

		await expect(connection.readMessage()).resolves.toEqual({
			type: "error",
			data: "game-not-active",
		})
	})

	it.concurrent("sends invalid-message websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")
		await expect(connection.readMessage()).resolves.toEqual(snapshotMessageMatcher())

		connection.webSocket.send("not json")

		await expect(connection.readMessage()).resolves.toEqual({
			type: "error",
			data: "invalid-message",
		})
	})

	it.concurrent("sends invalid-move-format websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")
		await expect(connection.readMessage()).resolves.toEqual(snapshotMessageMatcher())

		connection.webSocket.send(JSON.stringify({ type: "move", data: "e2e" }))

		await expect(connection.readMessage()).resolves.toEqual({
			type: "error",
			data: "invalid-move-format",
		})
	})

	it.concurrent("sends illegal-move websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e5" }))

		await expect(connections.white.readMessage()).resolves.toEqual({
			type: "error",
			data: "illegal-move",
		})
	})

	it.concurrent("sends not-your-turn websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.black.webSocket.send(JSON.stringify({ type: "move", data: "a7a6" }))

		await expect(connections.black.readMessage()).resolves.toEqual({
			type: "error",
			data: "not-your-turn",
		})
	})

	it.concurrent("closes websocket when receiving a binary message", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")
		const { webSocket } = connection
		const closeEventPromise = new Promise<CloseEvent>((resolve, reject) => {
			const timeout = setTimeout(
				() => reject(new Error("Timed out waiting for websocket close")),
				100,
			)

			webSocket.addEventListener(
				"close",
				(event) => {
					clearTimeout(timeout)
					resolve(event)
				},
				{ once: true },
			)
		})

		webSocket.send(new Uint8Array([1, 2, 3]))
		const event = await closeEventPromise

		expect(event.code).toBe(1003)
		expect(event.reason).toBe("binary messages are not supported")
	})
})
