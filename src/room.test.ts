import * as server from "@tanstack/react-start/server"
import { env } from "cloudflare:workers"
import { afterEach, beforeAll, describe, expect, inject, it, vi } from "vitest"

import { db } from "./db.server"
import {
	ensureRoomSessionMatches,
	ROOM_SESSION_COOKIE_NAME,
	roomSessionMiddleware,
	roomIdSchema,
	roomSessionCodec,
	type RoomSession,
} from "./room"
import { redirectToRoom } from "./room.server"
import { gamesTable } from "./schema.server"
import { expectRedirect, runInStartContext } from "./test-utils"

const ROOM_ID = roomIdSchema.parse("test-room-id-00000001")
const OTHER_ROOM_ID = roomIdSchema.parse("test-room-id-00000002")

beforeAll(async () => {
	await env.DB.exec(inject("TEST_SCHEMA_SQL"))
})

afterEach(async () => {
	await db.delete(gamesTable)
})

function roomSessionRequest(roomSession: RoomSession) {
	return {
		headers: {
			cookie: `${ROOM_SESSION_COOKIE_NAME}=${encodeURIComponent(roomSessionCodec.encode(roomSession))}`,
		},
	} as const satisfies RequestInit
}

async function runRoomSessionMiddleware(requestInit?: RequestInit) {
	return runInStartContext(
		() =>
			roomSessionMiddleware.options.server!({
				next: async (options?: unknown) => options,
			} as any),
		requestInit,
	)
}

describe("redirectToRoom", () => {
	it("redirects to the room from an existing session", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }
		using setCookieSpy = vi.spyOn(server, "setCookie")

		await expectRedirect(runInStartContext(redirectToRoom, roomSessionRequest(roomSession)), {
			to: "/$roomId",
			params: { roomId: roomSession.roomId },
		})
		expect(setCookieSpy).not.toHaveBeenCalled()
		await expect(db.select().from(gamesTable)).resolves.toEqual([])
	})

	it("creates a game and session cookie before redirecting to a new room", async () => {
		using setCookieSpy = vi.spyOn(server, "setCookie")

		const redirect = await runInStartContext(redirectToRoom)
		const games = await db.select().from(gamesTable).orderBy(gamesTable.id)
		expect(games).toMatchObject([{ white: expect.any(String), black: null }])
		const game = games[0]!

		await expectRedirect(redirect, {
			to: "/$roomId",
			params: { roomId: game.roomId },
		})
		expect(setCookieSpy).toHaveBeenCalledExactlyOnceWith(
			ROOM_SESSION_COOKIE_NAME,
			expect.any(String),
			expect.any(Object),
		)
		const roomSession = roomSessionCodec.decode(setCookieSpy.mock.calls[0]![1])
		expect(roomSession).toEqual({ roomId: game.roomId, token: game.white })
	})
})

describe("ensureRoomSessionMatches", () => {
	it("redirects home when there is no room session", async () => {
		await expectRedirect(
			runInStartContext(() => ensureRoomSessionMatches(ROOM_ID)),
			{
				to: "/",
			},
		)
	})

	it("redirects to the session room when the requested room differs", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }

		await expectRedirect(
			runInStartContext(
				() => ensureRoomSessionMatches(OTHER_ROOM_ID),
				roomSessionRequest(roomSession),
			),
			{
				to: "/$roomId",
				params: { roomId: roomSession.roomId },
			},
		)
	})

	it("allows matching room sessions", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }

		await expect(
			runInStartContext(() => ensureRoomSessionMatches(ROOM_ID), roomSessionRequest(roomSession)),
		).resolves.toBeUndefined()
	})
})

describe("roomSessionMiddleware", () => {
	it("deletes the cookie and throws unauthorized without a room session", async () => {
		using deleteCookieSpy = vi.spyOn(server, "deleteCookie")

		await expect(runRoomSessionMiddleware()).rejects.toThrow("Unauthorized")
		expect(deleteCookieSpy).toHaveBeenCalledExactlyOnceWith(ROOM_SESSION_COOKIE_NAME)
	})

	it("deletes the cookie and throws forbidden when no game matches the session", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }
		using deleteCookieSpy = vi.spyOn(server, "deleteCookie")

		await expect(runRoomSessionMiddleware(roomSessionRequest(roomSession))).rejects.toThrow(
			"Forbidden",
		)
		expect(deleteCookieSpy).toHaveBeenCalledExactlyOnceWith(ROOM_SESSION_COOKIE_NAME)
	})

	it("passes matching room sessions to the handler", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }
		using deleteCookieSpy = vi.spyOn(server, "deleteCookie")
		await db.insert(gamesTable).values({ roomId: roomSession.roomId, white: roomSession.token })

		await expect(runRoomSessionMiddleware(roomSessionRequest(roomSession))).resolves.toEqual(
			expect.objectContaining({
				context: { roomSession },
			}),
		)
		expect(deleteCookieSpy).not.toHaveBeenCalled()
	})
})
