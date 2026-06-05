import { env } from "cloudflare:workers"
import { eq } from "drizzle-orm"
import { afterAll, beforeAll, describe, expect, inject, it } from "vitest"

import { db } from "./db.server"
import {
	ensureRoomSessionMatches,
	ROOM_SESSION_COOKIE_NAME,
	roomSessionMiddleware,
	roomSessionCodec,
	type RoomId,
	type RoomSession,
} from "./room"
import { connectToRoomWebSocket, generateRoomId, redirectToRoom } from "./room.server"
import { gamesTable } from "./schema.server"
import { redirect, runInStartContext } from "./test-utils"

beforeAll(async () => {
	await env.DB.exec(inject("TEST_SCHEMA_SQL"))
})

const TEST_GAME_SERVER_ROOM_IDS = new Set<RoomId>()

afterAll(async () => {
	await Promise.all([
		...Array.from(TEST_GAME_SERVER_ROOM_IDS, (roomId) => env.GAME_SERVER.getByName(roomId).clear()),
		db.delete(gamesTable),
	])
})

function roomSessionRequest(roomSession: RoomSession) {
	return {
		headers: {
			cookie: `${ROOM_SESSION_COOKIE_NAME}=${encodeURIComponent(roomSessionCodec.encode(roomSession))}`,
		},
	} as const satisfies RequestInit
}

describe("redirectToRoom", () => {
	it.concurrent("redirects to the room from an existing session", async () => {
		const roomSession = { roomId: generateRoomId(), token: "white-token" }
		const { result, response } = await runInStartContext(
			redirectToRoom,
			roomSessionRequest(roomSession),
		)

		expect(result).toEqual(redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } }))
		expect(response.headers.has("Set-Cookie")).toBe(false)

		const games = await db
			.select()
			.from(gamesTable)
			.where(eq(gamesTable.roomId, roomSession.roomId))
		expect(games).toEqual([])
	})

	it.concurrent("creates a game and session cookie before redirecting to a new room", async () => {
		const { result, response } = await runInStartContext(redirectToRoom)

		const cookie = response.headers.get("Set-Cookie")
		expect(cookie).toContain(`${ROOM_SESSION_COOKIE_NAME}=`)
		const roomSession = roomSessionCodec.decode(
			decodeURIComponent(cookie!.split(`${ROOM_SESSION_COOKIE_NAME}=`)[1]!.split(";")[0]!),
		)

		const games = await db
			.select()
			.from(gamesTable)
			.where(eq(gamesTable.roomId, roomSession.roomId))
		expect(games).toMatchObject([{ white: expect.any(String), black: null }])

		const game = games[0]!
		expect(result).toEqual(redirect({ to: "/$roomId", params: { roomId: game.roomId } }))
		expect(response.headers.get("Set-Cookie")).toContain("Max-Age=1800")
		expect(response.headers.get("Set-Cookie")).toContain("Max-Age=1800")
		expect(response.headers.get("Set-Cookie")).toContain("Secure")
		expect(response.headers.get("Set-Cookie")).toContain("SameSite=Lax")
		expect(roomSession).toEqual({ roomId: game.roomId, token: game.white })
	})
})

describe("ensureRoomSessionMatches", () => {
	it.concurrent("redirects home when there is no room session", async () => {
		const { result } = await runInStartContext(() => ensureRoomSessionMatches(generateRoomId()))

		expect(result).toEqual(redirect({ to: "/" }))
	})

	it.concurrent("redirects to the session room when the requested room differs", async () => {
		const roomSession = { roomId: generateRoomId(), token: "white-token" }
		const { result } = await runInStartContext(
			() => ensureRoomSessionMatches(generateRoomId()),
			roomSessionRequest(roomSession),
		)

		expect(result).toEqual(redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } }))
	})

	it.concurrent("allows matching room sessions", async () => {
		const roomSession = { roomId: generateRoomId(), token: "white-token" }
		const { result } = await runInStartContext(
			() => ensureRoomSessionMatches(roomSession.roomId),
			roomSessionRequest(roomSession),
		)

		expect(result).toBeUndefined()
	})
})

function runRoomSessionMiddleware() {
	return roomSessionMiddleware.options.server!({
		next: async (options?: unknown) => options,
	} as any)
}

describe("roomSessionMiddleware", () => {
	it.concurrent("deletes the cookie and throws unauthorized without a room session", async () => {
		const { error, response } = await runInStartContext(runRoomSessionMiddleware)

		expect(error).toEqual(new Error("Unauthorized"))
		expect(response.headers.get("Set-Cookie")).toContain(`${ROOM_SESSION_COOKIE_NAME}=`)
		expect(response.headers.get("Set-Cookie")).toContain("Max-Age=0")
	})

	it.concurrent("deletes the cookie and throws forbidden when no game matches the session", async () => {
		const roomSession = { roomId: generateRoomId(), token: "white-token" }
		const { error, response } = await runInStartContext(
			runRoomSessionMiddleware,
			roomSessionRequest(roomSession),
		)

		expect(error).toEqual(new Error("Forbidden"))
		expect(response.headers.get("Set-Cookie")).toContain(`${ROOM_SESSION_COOKIE_NAME}=`)
		expect(response.headers.get("Set-Cookie")).toContain("Max-Age=0")
	})

	it.concurrent("does not delete the cookie for matching room sessions", async () => {
		const roomId = generateRoomId()
		const roomSession = { roomId, token: "white-token" }
		await db.insert(gamesTable).values({ roomId, white: roomSession.token })
		const { response } = await runInStartContext(
			runRoomSessionMiddleware,
			roomSessionRequest(roomSession),
		)

		expect(response.headers.has("Set-Cookie")).toBe(false)
	})

	it.concurrent("passes matching room sessions to the handler", async () => {
		const roomId = generateRoomId()
		const whiteRoomSession = { roomId, token: "white-token" }
		const blackRoomSession = { roomId, token: "black-token" }
		await db.insert(gamesTable).values({
			roomId: whiteRoomSession.roomId,
			white: whiteRoomSession.token,
			black: blackRoomSession.token,
		})
		const [{ result: whiteResult }, { result: blackResult }] = await Promise.all([
			runInStartContext(runRoomSessionMiddleware, roomSessionRequest(whiteRoomSession)),
			runInStartContext(runRoomSessionMiddleware, roomSessionRequest(blackRoomSession)),
		])

		expect(whiteResult).toEqual(
			expect.objectContaining({
				context: { roomSession: whiteRoomSession, player: "white" },
			}),
		)
		expect(blackResult).toEqual(
			expect.objectContaining({
				context: { roomSession: blackRoomSession, player: "black" },
			}),
		)
	})
})

function connectToRoomWebSocketRequest(
	roomId: RoomId,
	roomSession: RoomSession,
	requestInit?: RequestInit,
) {
	return connectToRoomWebSocket(
		new Request("https://chess.localhost", requestInit),
		roomSession,
		"white",
		roomId,
	)
}

describe("connectToRoomWebSocket", () => {
	it.concurrent("returns upgrade required for non-websocket requests", async () => {
		const roomId = generateRoomId()
		const response = await connectToRoomWebSocketRequest(roomId, {
			roomId,
			token: "white-token",
		})

		expect(response.status).toBe(426)
		expect(response.headers.get("Upgrade")).toBe("websocket")
	})

	it.concurrent("returns forbidden when the requested room differs from the session room", async () => {
		const roomId = generateRoomId()
		const response = await connectToRoomWebSocketRequest(
			roomId,
			{ roomId: generateRoomId(), token: "white-token" },
			{
				headers: { Upgrade: "websocket" },
			},
		)

		expect(response.status).toBe(403)
	})

	it.concurrent("forwards matching room websocket requests with the player header", async () => {
		const roomId = generateRoomId()
		const response = await connectToRoomWebSocketRequest(
			roomId,
			{ roomId, token: "white-token" },
			{
				headers: { Upgrade: "websocket" },
			},
		)

		expect(response.status).toBe(101)
		expect(response.webSocket).toBeInstanceOf(WebSocket)

		response.webSocket!.accept()
		response.webSocket!.close()
		TEST_GAME_SERVER_ROOM_IDS.add(roomId)
	})
})
