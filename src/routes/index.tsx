import { createFileRoute } from "@tanstack/react-router"

import { redirectToRoom } from "#/room"

export const Route = createFileRoute("/")({
	server: {
		handlers: {
			GET: () => redirectToRoom(),
		},
	},
})
