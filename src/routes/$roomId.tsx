import { createFileRoute } from "@tanstack/react-router"
import z from "zod"

import { Chessboard } from "#/chessboard"
import { validateRoomSession } from "#/room"
import { roomIdSchema } from "#/room"

const paramsSchema = z.object({ roomId: roomIdSchema })

export const Route = createFileRoute("/$roomId")({
	params: {
		parse: (params) => {
			const { data, success } = paramsSchema.safeParse(params)
			if (!success) return false
			return data
		},
	},
	beforeLoad: async ({ params }) => {
		await validateRoomSession({ data: params.roomId })
	},
	component: RouteComponent,
})

function RouteComponent() {
	return (
		<div className="h-full content-center">
			<Chessboard />
		</div>
	)
}
