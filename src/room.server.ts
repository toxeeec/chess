import { redirect } from "@tanstack/react-router"
import { createServerFn } from "@tanstack/react-start"
import { setCookie } from "@tanstack/react-start/server"
import { env } from "cloudflare:workers"
import { and, eq, isNull, or, sql } from "drizzle-orm"
import { z } from "zod"

import { db } from "./db.server"
import { nanoid } from "./nanoid.server"
import {
	getRoomSessionFromCookie,
	ROOM_SESSION_COOKIE_NAME,
	type RoomId,
	roomIdSchema,
	roomSessionCodec,
	type RoomSession,
	type Player,
} from "./room"
import { gamesTable } from "./schema.server"

const PLAYER_HEADER = "Player-Color"
const GAME_CONFIG = {
	joinTimeoutMs: 2 * 60_000,
	firstMoveTimeoutMs: 30_000,
	disconnectTimeoutMs: 60_000,
	timeControlMs: 5 * 60_000,
} as const

export const redirectToRoom = createServerFn().handler(async () => {
	let roomSession = getRoomSessionFromCookie()
	if (roomSession && (await isResumableRoomSession(roomSession))) {
		throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })
	}

	roomSession = await createRoomSession()
	setRoomSessionCookie(roomSession)
	throw redirect({ to: "/$roomId", params: { roomId: roomSession.roomId } })
})

export const joinRoomFromInvite = createServerFn()
	.validator(z.object({ roomId: roomIdSchema }))
	.handler(async ({ data: { roomId } }) => {
		const currentRoomSession = getRoomSessionFromCookie()
		if (currentRoomSession && (await isResumableRoomSession(currentRoomSession))) {
			throw redirect({ to: "/$roomId", params: { roomId: currentRoomSession.roomId } })
		}

		const roomSession = { roomId, token: generateToken() }

		const [game] = await db
			.update(gamesTable)
			.set({
				white: sql`coalesce(${gamesTable.white}, ${roomSession.token})`,
				black: sql`case when ${gamesTable.white} is not null then ${roomSession.token} else ${gamesTable.black} end`,
			})
			.where(
				and(eq(gamesTable.roomId, roomId), or(isNull(gamesTable.white), isNull(gamesTable.black))),
			)
			.returning()

		if (!game) throw redirect({ to: "/" })

		setRoomSessionCookie(roomSession)
		throw redirect({ to: "/$roomId", params: { roomId } })
	})

function setRoomSessionCookie(roomSession: RoomSession) {
	setCookie(ROOM_SESSION_COOKIE_NAME, roomSessionCodec.encode(roomSession), {
		maxAge: 30 * 60, // 30 minutes
		secure: true,
		sameSite: "lax",
	})
}

async function createRoomSession() {
	const roomSession = { roomId: generateRoomId(), token: generateToken() }
	await db.insert(gamesTable).values({ roomId: roomSession.roomId, white: roomSession.token })
	await env.GAME_SERVER.getByName(roomSession.roomId).init(GAME_CONFIG)

	return roomSession
}

async function isResumableRoomSession(roomSession: RoomSession) {
	const game = await getGameByRoomSession(roomSession)
	if (!game) return false

	const { status } = await env.GAME_SERVER.getByName(roomSession.roomId).snapshot()
	return status === "waiting" || status === "active"
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
	player: Player,
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
