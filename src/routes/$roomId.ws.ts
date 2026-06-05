import { createFileRoute } from "@tanstack/react-router"

import { roomSessionMiddleware } from "#/room"
import { connectToRoomWebSocket } from "#/room.server"

import { parseRoomParams } from "./-room"

export const Route = createFileRoute("/$roomId/ws")({
	params: {
		parse: parseRoomParams,
	},
	server: {
		middleware: [roomSessionMiddleware],
		handlers: {
			GET: ({ request, context: { roomSession, player }, params: { roomId } }) =>
				connectToRoomWebSocket(request, roomSession, player, roomId),
		},
	},
})
