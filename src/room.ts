import { redirect } from "@tanstack/react-router"
import { createIsomorphicFn, createMiddleware, createServerFn } from "@tanstack/react-start"
import { deleteCookie, getCookie } from "@tanstack/react-start/server"
import { env } from "cloudflare:workers"
import { z } from "zod"

import { getGameByRoomSession } from "./room.server"
import { jsonCodec } from "./zod"

export const ROOM_SESSION_COOKIE_NAME = "room-session"

export const roomIdSchema = z.nanoid().brand<"RoomId">()
export type RoomId = z.infer<typeof roomIdSchema>

export const roomSessionCodec = jsonCodec(z.object({ token: z.string(), roomId: roomIdSchema }))
export type RoomSession = z.infer<typeof roomSessionCodec>

export const getRoomSessionFromCookie = createIsomorphicFn()
	.server(() => {
		const cookie = getCookie(ROOM_SESSION_COOKIE_NAME)
		if (!cookie) return undefined
		return roomSessionCodec.safeDecode(cookie).data
	})
	.client(() => {
		const cookie = document.cookie
			.split("; ")
			.find((cookie) => cookie.startsWith(`${ROOM_SESSION_COOKIE_NAME}=`))
			?.slice(ROOM_SESSION_COOKIE_NAME.length + 1)
		if (!cookie) return undefined
		return roomSessionCodec.safeDecode(decodeURIComponent(cookie)).data
	})

export function ensureRoomSessionMatches(roomId: RoomId) {
	const roomSession = getRoomSessionFromCookie()
	if (!roomSession) throw redirect({ to: "/" })
	if (roomSession.roomId !== roomId) {
		throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })
	}
}

export const roomSessionMiddleware = createMiddleware().server(async ({ next }) => {
	const roomSession = getRoomSessionFromCookie()
	if (!roomSession) {
		deleteCookie(ROOM_SESSION_COOKIE_NAME)
		throw new Error("Unauthorized")
	}

	const game = await getGameByRoomSession(roomSession)

	if (!game) {
		deleteCookie(ROOM_SESSION_COOKIE_NAME)
		throw new Error("Forbidden")
	}

	const player = roomSession.token === game.white ? ("white" as const) : ("black" as const)

	return next({ context: { roomSession, player } })
})

export const getGameState = createServerFn()
	.middleware([roomSessionMiddleware])
	.handler(async ({ context }) => {
		const { revision, fen, legalMoves } = await env.GAME_SERVER.getByName(
			context.roomSession.roomId,
		).snapshot()
		return { revision, fen, legalMoves }
	})
