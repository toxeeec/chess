import { redirect } from "@tanstack/react-router"
import { createServerFn } from "@tanstack/react-start"
import { setCookie } from "@tanstack/react-start/server"
import { and, eq, or, sql } from "drizzle-orm"

import { db } from "./db.server"
import { nanoid } from "./nanoid.server"
import {
	getRoomSessionFromCookie,
	ROOM_SESSION_COOKIE_NAME,
	roomIdSchema,
	roomSessionCodec,
	type RoomSession,
} from "./room"
import { gamesTable } from "./schema.server"

export const redirectToRoom = createServerFn().handler(async () => {
	let roomSession = getRoomSessionFromCookie()
	if (roomSession) throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })

	roomSession = await createRoomSession()
	setCookie(ROOM_SESSION_COOKIE_NAME, roomSessionCodec.encode(roomSession), {
		maxAge: 30 * 60, // 30 minutes
		secure: true,
		sameSite: "lax",
	})
	throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })
})

async function createRoomSession() {
	const roomSession = { roomId: generateRoomId(), token: generateToken() }
	await db.insert(gamesTable).values({ roomId: roomSession.roomId, white: roomSession.token })

	return roomSession
}

export async function hasGameForSession(roomSession: RoomSession) {
	const [exists] = await db
		.select({ exists: sql`1` })
		.from(gamesTable)
		.where(
			and(
				eq(gamesTable.roomId, roomSession.roomId),
				or(eq(gamesTable.white, roomSession.token), eq(gamesTable.black, roomSession.token)),
			),
		)
		.limit(1)

	return Boolean(exists)
}

function generateRoomId() {
	return roomIdSchema.parse(nanoid())
}

function generateToken() {
	return Array.from(crypto.getRandomValues(new Uint8Array(32)), (byte) =>
		byte.toString(16).padStart(2, "0"),
	).join("")
}
