import { createFileRoute } from "@tanstack/react-router"
import { z } from "zod"

import { roomIdSchema, roomSessionMiddleware } from "#/room"
import { connectToRoomWebSocket } from "#/room.server"

const paramsSchema = z.object({ roomId: roomIdSchema })

export const Route = createFileRoute("/$roomId/ws")({
	params: {
		parse: (params) => {
			const { data, success } = paramsSchema.safeParse(params)
			if (!success) return false
			return data
		},
	},
	server: {
		middleware: [roomSessionMiddleware],
		handlers: {
			GET: ({ request, context: { roomSession, player }, params: { roomId } }) =>
				connectToRoomWebSocket(request, roomSession, player, roomId),
		},
	},
})
