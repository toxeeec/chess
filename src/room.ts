import { redirect } from "@tanstack/react-router"
import { createIsomorphicFn, createMiddleware, createServerFn } from "@tanstack/react-start"
import { deleteCookie, getCookie } from "@tanstack/react-start/server"
import { env } from "cloudflare:workers"
import z from "zod"

import { hasGameForSession } from "./room.server"

export const ROOM_SESSION_COOKIE_NAME = "room-session"

export const roomIdSchema = z.nanoid().brand<"RoomId">()
export type RoomId = z.infer<typeof roomIdSchema>

export const roomSessionCodec = z.codec(
	z.string(),
	z.object({ token: z.string(), roomId: roomIdSchema }),
	{
		decode: (jsonString, ctx) => {
			try {
				// oxlint-disable-next-line
				return JSON.parse(jsonString) as any
			} catch (err) {
				ctx.issues.push({
					code: "invalid_format",
					format: "json",
					input: jsonString,
					message: String(err),
				})
				return z.NEVER
			}
		},
		encode: (value) => JSON.stringify(value),
	},
)
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

	if (!(await hasGameForSession(roomSession))) {
		deleteCookie(ROOM_SESSION_COOKIE_NAME)
		throw new Error("Forbidden")
	}

	return next({ context: { roomSession } })
})

export const getGameFen = createServerFn()
	.middleware([roomSessionMiddleware])
	.handler(async ({ context }) => {
		return env.GAME_SERVER.getByName(context.roomSession.roomId).fen()
	})
