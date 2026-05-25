import * as server from "@tanstack/react-start/server"
import { env } from "cloudflare:workers"
import { afterEach, beforeAll, describe, expect, inject, it, vi } from "vitest"
import z from "zod"

import { db } from "./db.server"
import { redirectToRoom, roomIdSchema, validateRoomSession } from "./room"
import { ROOM_SESSION_COOKIE_NAME, roomSessionCodec } from "./room.server"
import { gamesTable } from "./schema.server"
import { expectRedirect, runTestServerFn } from "./test-utils"

type RoomSession = z.infer<typeof roomSessionCodec>

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
	}
}

describe("redirectToRoom", () => {
	it("redirects to the room from an existing session", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }
		using setCookieSpy = vi.spyOn(server, "setCookie")

		await expectRedirect(runTestServerFn(redirectToRoom, roomSessionRequest(roomSession)), {
			to: "/$roomId",
			params: { roomId: roomSession.roomId },
		})
		expect(setCookieSpy).not.toHaveBeenCalled()
		await expect(db.select().from(gamesTable)).resolves.toEqual([])
	})

	it("creates a game and session cookie before redirecting to a new room", async () => {
		using setCookieSpy = vi.spyOn(server, "setCookie")

		const redirect = await runTestServerFn(redirectToRoom)
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

describe("validateRoomSession", () => {
	it("redirects home when there is no room session", async () => {
		await expectRedirect(
			runTestServerFn(() => validateRoomSession({ data: ROOM_ID })),
			{
				to: "/",
			},
		)
	})

	it("redirects to the session room when validating another room", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }

		await expectRedirect(
			runTestServerFn(
				() => validateRoomSession({ data: OTHER_ROOM_ID }),
				roomSessionRequest(roomSession),
			),
			{
				to: "/$roomId",
				params: { roomId: roomSession.roomId },
			},
		)
	})

	it("allows valid room sessions for the player", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }
		await db.insert(gamesTable).values({ roomId: roomSession.roomId, white: roomSession.token })
		using deleteCookieSpy = vi.spyOn(server, "deleteCookie")

		await expect(
			runTestServerFn(
				() => validateRoomSession({ data: roomSession.roomId }),
				roomSessionRequest(roomSession),
			),
		).resolves.toBeUndefined()
		expect(deleteCookieSpy).not.toHaveBeenCalled()
	})

	it("deletes stale room sessions when the game does not exist", async () => {
		const roomSession = { roomId: ROOM_ID, token: "white-token" }
		using deleteCookieSpy = vi.spyOn(server, "deleteCookie")

		await expectRedirect(
			runTestServerFn(
				() => validateRoomSession({ data: roomSession.roomId }),
				roomSessionRequest(roomSession),
			),
			{ to: "/" },
		)
		expect(deleteCookieSpy).toHaveBeenCalledWith(ROOM_SESSION_COOKIE_NAME)
	})

	it("deletes stale room sessions when the token does not match", async () => {
		const roomSession = { roomId: ROOM_ID, token: "other-token" }
		await db
			.insert(gamesTable)
			.values({ roomId: roomSession.roomId, white: "white-token", black: "black-token" })
		using deleteCookieSpy = vi.spyOn(server, "deleteCookie")

		await expectRedirect(
			runTestServerFn(
				() => validateRoomSession({ data: roomSession.roomId }),
				roomSessionRequest(roomSession),
			),
			{ to: "/" },
		)
		expect(deleteCookieSpy).toHaveBeenCalledWith(ROOM_SESSION_COOKIE_NAME)
	})
})
