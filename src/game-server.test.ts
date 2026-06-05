import { env } from "cloudflare:workers"
import { afterAll, describe, expect, it } from "vitest"

import type { Player, RoomId } from "./room"
import { generateRoomId } from "./room.server"

const TEST_ROOM_IDS = new Set<RoomId>()

afterAll(async () => {
	await Promise.all(
		Array.from(TEST_ROOM_IDS, (roomId) => env.GAME_SERVER.getByName(roomId).clear()),
	)
})

function generateTestRoomId() {
	const roomId = generateRoomId()
	TEST_ROOM_IDS.add(roomId)
	return roomId
}

function waitForWebSocketMessage(webSocket: WebSocket) {
	return new Promise<unknown>((resolve, reject) => {
		const timeout = setTimeout(() => reject(new Error("Timed out waiting for message")), 100)

		webSocket.addEventListener(
			"message",
			(event) => {
				clearTimeout(timeout)
				expect(event.data).toBeTypeOf("string")
				resolve(JSON.parse(event.data))
			},
			{ once: true },
		)
	})
}

async function acceptWebSocketWithInitialMessage(roomId: RoomId, player: Player) {
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

	const webSocket = Object.assign(response.webSocket!, {
		[Symbol.dispose]() {
			webSocket.close()
		},
	})
	webSocket.accept()
	const initialMessage = await waitForWebSocketMessage(webSocket)
	return { initialMessage, webSocket }
}

describe("GameServer", () => {
	it.concurrent("returns forbidden when the player header is missing", async () => {
		const roomId = generateTestRoomId()
		const response = await env.GAME_SERVER.getByName(roomId).fetch(
			new Request("https://chess.localhost", {
				headers: { Upgrade: "websocket" },
			}),
		)

		expect(response.status).toBe(403)
	})

	it.concurrent("sends an initial snapshot to connected sockets", async () => {
		const roomId = generateTestRoomId()
		const { webSocket, initialMessage } = await acceptWebSocketWithInitialMessage(roomId, "white")

		expect(initialMessage).toEqual({
			type: "snapshot",
			data: {
				revision: 0,
				fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
				legalMoves: expect.any(String),
			},
		})
		webSocket.close()
	})

	it.concurrent("broadcasts moves to connected room sockets", async () => {
		const roomId = generateTestRoomId()
		const [whiteConnection, blackConnection] = await Promise.all([
			acceptWebSocketWithInitialMessage(roomId, "white"),
			acceptWebSocketWithInitialMessage(roomId, "black"),
		])
		using whiteWs = whiteConnection.webSocket
		using blackWs = blackConnection.webSocket

		const whiteMovePromise = waitForWebSocketMessage(whiteWs)
		const blackMovePromise = waitForWebSocketMessage(blackWs)

		whiteWs.send(JSON.stringify({ type: "move", data: "e2e3" }))

		const [whiteMove, blackMove] = await Promise.all([whiteMovePromise, blackMovePromise])
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

	it.concurrent("persists moved game state for new websocket connections", async () => {
		const roomId = generateTestRoomId()
		const connection = await acceptWebSocketWithInitialMessage(roomId, "white")
		using webSocket = connection.webSocket

		const moveMessagePromise = waitForWebSocketMessage(webSocket)
		webSocket.send(JSON.stringify({ type: "move", data: "e2e3" }))
		await moveMessagePromise

		const reconnectedConnection = await acceptWebSocketWithInitialMessage(roomId, "black")
		using reconnected = reconnectedConnection.webSocket
		void reconnected

		expect(reconnectedConnection.initialMessage).toEqual({
			type: "snapshot",
			data: {
				revision: 1,
				fen: "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b - - 0 1",
				legalMoves: expect.any(String),
			},
		})
	})

	it.concurrent("sends websocket protocol errors", async () => {
		const roomId = generateTestRoomId()

		const expectError = async (message: string, error: string, player: Player) => {
			using webSocket = (await acceptWebSocketWithInitialMessage(roomId, player)).webSocket

			webSocket.send(message)
			expect(await waitForWebSocketMessage(webSocket)).toEqual({
				type: "error",
				data: error,
			})
		}

		await Promise.all([
			expectError("not json", "invalid-message", "white"),
			expectError(JSON.stringify({ type: "move", data: "e2e" }), "invalid-move-format", "white"),
			expectError(JSON.stringify({ type: "move", data: "e2e5" }), "illegal-move", "white"),
			expectError(JSON.stringify({ type: "move", data: "a7a6" }), "not-your-turn", "black"),
		])
	})

	it.concurrent("closes binary websocket messages", async () => {
		const roomId = generateTestRoomId()
		const connection = await acceptWebSocketWithInitialMessage(roomId, "white")
		using webSocket = connection.webSocket
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
