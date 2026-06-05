import { createFileRoute } from "@tanstack/react-router"

import { joinRoomFromInvite } from "#/room.server"

import { parseRoomParams } from "./-room"

export const Route = createFileRoute("/$roomId/invite")({
	params: {
		parse: parseRoomParams,
	},
	server: {
		handlers: {
			GET: ({ params: { roomId } }) => joinRoomFromInvite({ data: { roomId } }),
		},
	},
})
