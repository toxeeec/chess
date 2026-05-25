import { redirect } from "@tanstack/react-router"
import { createServerFn } from "@tanstack/react-start"
import { deleteCookie, setCookie } from "@tanstack/react-start/server"
import z from "zod"

import {
	createRoomSession,
	getRoomSession,
	hasGameForSession,
	ROOM_SESSION_COOKIE_NAME,
	roomSessionCodec,
} from "./room.server"

export const roomIdSchema = z.nanoid().brand<"RoomId">()
export type RoomId = z.infer<typeof roomIdSchema>

export const redirectToRoom = createServerFn({ method: "GET" }).handler(async () => {
	let roomSession = getRoomSession()
	if (roomSession) throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })

	roomSession = await createRoomSession()
	setCookie(ROOM_SESSION_COOKIE_NAME, roomSessionCodec.encode(roomSession), {
		maxAge: 30 * 60, // 30 minutes
		secure: true,
		sameSite: "lax",
	})
	throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })
})

export const validateRoomSession = createServerFn()
	.inputValidator(roomIdSchema)
	.handler(async ({ data: roomId }) => {
		const roomSession = getRoomSession()
		if (!roomSession) throw redirect({ to: "/" })
		if (roomId !== roomSession.roomId) {
			throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })
		}

		if (await hasGameForSession(roomSession)) return

		deleteCookie(ROOM_SESSION_COOKIE_NAME)
		throw redirect({ to: "/" })
	})
