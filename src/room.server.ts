import { getCookie } from "@tanstack/react-start/server"
import { and, eq, or, sql } from "drizzle-orm"
import z from "zod"

import { db } from "./db.server"
import { nanoid } from "./nanoid.server"
import { roomIdSchema } from "./room"
import { gamesTable } from "./schema.server"

export const ROOM_SESSION_COOKIE_NAME = "room-session"

export const roomSessionCodec = z.codec(
	z.string(),
	z.object({ token: z.string(), roomId: roomIdSchema }),
	{
		decode: (jsonString, ctx) => {
			try {
				return JSON.parse(jsonString)
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

type RoomSession = z.infer<typeof roomSessionCodec>

export function getRoomSession() {
	const roomSessionCookie = getCookie(ROOM_SESSION_COOKIE_NAME)
	if (!roomSessionCookie) return null

	const { success, data } = roomSessionCodec.safeDecode(roomSessionCookie)
	if (!success) return null
	return data
}

export async function createRoomSession() {
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
