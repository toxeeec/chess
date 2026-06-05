import { redirect } from "@tanstack/react-router"
import { createServerFn } from "@tanstack/react-start"
import { setCookie } from "@tanstack/react-start/server"
import { env } from "cloudflare:workers"
import { and, eq, or } from "drizzle-orm"

import { db } from "./db.server"
import { nanoid } from "./nanoid.server"
import {
	getRoomSessionFromCookie,
	ROOM_SESSION_COOKIE_NAME,
	type RoomId,
	roomIdSchema,
	roomSessionCodec,
	type RoomSession,
} from "./room"
import { gamesTable } from "./schema.server"

const PLAYER_HEADER = "Player-Color"

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

export async function getGameByRoomSession(roomSession: RoomSession) {
	const [game] = await db
		.select()
		.from(gamesTable)
		.where(
			and(
				eq(gamesTable.roomId, roomSession.roomId),
				or(eq(gamesTable.white, roomSession.token), eq(gamesTable.black, roomSession.token)),
			),
		)
		.limit(1)

	return game
}

export function connectToRoomWebSocket(
	request: Request,
	roomSession: RoomSession,
	player: "white" | "black",
	roomId: RoomId,
) {
	if (request.headers.get("Upgrade")?.toLowerCase() !== "websocket") {
		return new Response(null, {
			status: 426,
			headers: { Upgrade: "websocket" },
		})
	}

	if (roomSession.roomId !== roomId) {
		return new Response(null, { status: 403 })
	}

	const headers = new Headers(request.headers)
	headers.set(PLAYER_HEADER, player)

	return env.GAME_SERVER.getByName(roomId).fetch(new Request(request, { headers }))
}

export function generateRoomId() {
	return roomIdSchema.parse(nanoid())
}

function generateToken() {
	return Array.from(crypto.getRandomValues(new Uint8Array(32)), (byte) =>
		byte.toString(16).padStart(2, "0"),
	).join("")
}
