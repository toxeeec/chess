import { runDurableObjectAlarm } from "cloudflare:test"
import { env } from "cloudflare:workers"
import { describe, expect, it } from "vitest"

import type { Player, RoomId } from "./room"
import { generateRoomId } from "./room.server"
import { TEST_GAME_CONFIG } from "./test-utils"

function sleep(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms))
}

function expectSnapshotMessage(messagePromise: Promise<unknown>) {
	return expect(messagePromise).resolves.toEqual({
		type: "snapshot",
		data: expect.objectContaining({
			fen: expect.any(String),
			revision: expect.any(Number),
			status: expect.any(String),
		}),
	})
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
		expectSnapshotMessage(white.readMessage()),
		expectSnapshotMessage(black.readMessage()),
	])
	const [whiteStatusMessage, blackStatusMessage] = await Promise.all([
		white.readMessage(),
		black.readMessage(),
	])

	expect(whiteStatusMessage).toEqual({
		type: "status",
		data: { status: "active", legalMoves: expect.any(String) },
	})
	expect(blackStatusMessage).toEqual({
		type: "status",
		data: { status: "active", legalMoves: expect.any(String) },
	})

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
		await expectSnapshotMessage(whiteConnection.readMessage())

		await sleep(10)
		await runDurableObjectAlarm(stub)

		expect(await whiteConnection.readMessage()).toEqual({
			type: "status",
			data: { status: "expired", legalMoves: "" },
		})
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

		await Promise.all([
			expect(connections.white.readMessage()).resolves.toEqual({
				type: "status",
				data: { status: "expired", legalMoves: "" },
			}),
			expect(connections.black.readMessage()).resolves.toEqual({
				type: "status",
				data: { status: "expired", legalMoves: "" },
			}),
		])
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

		await expect(connections.black.readMessage()).resolves.toEqual({
			type: "status",
			data: { status: "expired", legalMoves: "" },
		})
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

		await expect(connections.black.readMessage()).resolves.toEqual({
			type: "status",
			data: { status: "ended", legalMoves: "" },
		})
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

		await expect(connection.readMessage()).resolves.toEqual({
			type: "snapshot",
			data: {
				revision: 0,
				fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
				status: "waiting",
				legalMoves: "",
			},
		})
	})

	it.concurrent("broadcasts moves to connected room sockets", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))

		const [whiteMove, blackMove] = await Promise.all([
			connections.white.readMessage(),
			connections.black.readMessage(),
		])
		const expectedMoveMessage = {
			type: "move",
			data: {
				revision: 1,
				move: "e2e3",
				turn: "black",
				legalMoves: expect.any(String),
			},
		}

		expect(whiteMove).toEqual(expectedMoveMessage)
		expect(blackMove).toEqual(expectedMoveMessage)
	})

	it.concurrent("sends latest game snapshot to new websocket connections", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))

		using newConnection = await acceptWebSocket(roomId, "black")

		await expect(newConnection.readMessage()).resolves.toEqual({
			type: "snapshot",
			data: {
				revision: 1,
				fen: "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b - - 0 1",
				status: "active",
				legalMoves: expect.any(String),
			},
		})
	})

	it.concurrent("sends expired snapshots to new connections after join timeout", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			joinTimeoutMs: 1,
		})
		using whiteConnection = await acceptWebSocket(roomId, "white")
		await expectSnapshotMessage(whiteConnection.readMessage())

		await sleep(10)
		await runDurableObjectAlarm(stub)
		expect(await whiteConnection.readMessage()).toEqual({
			type: "status",
			data: { status: "expired", legalMoves: "" },
		})

		using blackConnection = await acceptWebSocket(roomId, "black")
		await expect(blackConnection.readMessage()).resolves.toEqual({
			type: "snapshot",
			data: {
				revision: 0,
				fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
				status: "expired",
				legalMoves: "",
			},
		})
	})

	it.concurrent("sends expired snapshots to new connections after first move timeout", async () => {
		const roomId = generateRoomId()
		const stub = env.GAME_SERVER.getByName(roomId)
		await stub.init({
			...TEST_GAME_CONFIG,
			firstMoveTimeoutMs: 1,
		})
		using connections = await connectPlayers(roomId)

		await sleep(10)
		await runDurableObjectAlarm(stub)

		await Promise.all([
			expect(connections.white.readMessage()).resolves.toEqual({
				type: "status",
				data: { status: "expired", legalMoves: "" },
			}),
			expect(connections.black.readMessage()).resolves.toEqual({
				type: "status",
				data: { status: "expired", legalMoves: "" },
			}),
		])

		using newConnection = await acceptWebSocket(roomId, "white")
		await expect(newConnection.readMessage()).resolves.toEqual({
			type: "snapshot",
			data: {
				revision: 0,
				fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
				status: "expired",
				legalMoves: "",
			},
		})
	})

	it.concurrent("rejects moves before both players join", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")
		await expectSnapshotMessage(connection.readMessage())

		connection.webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))

		expect(await connection.readMessage()).toEqual({
			type: "error",
			data: "game-not-active",
		})
	})

	it.concurrent("sends invalid-message websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")
		await expectSnapshotMessage(connection.readMessage())

		connection.webSocket.send("not json")

		expect(await connection.readMessage()).toEqual({
			type: "error",
			data: "invalid-message",
		})
	})

	it.concurrent("sends invalid-move-format websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connection = await acceptWebSocket(roomId, "white")
		await expectSnapshotMessage(connection.readMessage())

		connection.webSocket.send(JSON.stringify({ type: "move", data: "e2e" }))

		expect(await connection.readMessage()).toEqual({
			type: "error",
			data: "invalid-move-format",
		})
	})

	it.concurrent("sends illegal-move websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.white.webSocket.send(JSON.stringify({ type: "move", data: "e2e5" }))

		expect(await connections.white.readMessage()).toEqual({
			type: "error",
			data: "illegal-move",
		})
	})

	it.concurrent("sends not-your-turn websocket errors", async () => {
		const roomId = generateRoomId()
		await env.GAME_SERVER.getByName(roomId).init(TEST_GAME_CONFIG)
		using connections = await connectPlayers(roomId)

		connections.black.webSocket.send(JSON.stringify({ type: "move", data: "a7a6" }))

		expect(await connections.black.readMessage()).toEqual({
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
