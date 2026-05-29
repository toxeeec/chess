import { createFileRoute } from "@tanstack/react-router"

import { redirectToRoom } from "#/room.server"

export const Route = createFileRoute("/")({
	server: {
		handlers: {
			GET: () => redirectToRoom(),
		},
	},
})
