import { createFileRoute } from "@tanstack/react-router"
import z from "zod"

import { Chessboard } from "#/chessboard"
import { getGameFen, ensureRoomSessionMatches, roomIdSchema } from "#/room"

const paramsSchema = z.object({ roomId: roomIdSchema })

export const Route = createFileRoute("/$roomId")({
	params: {
		parse: (params) => {
			const { data, success } = paramsSchema.safeParse(params)
			if (!success) return false
			return data
		},
	},
	beforeLoad: ({ params }) => ensureRoomSessionMatches(params.roomId),
	loader: () => getGameFen(),
	component: RouteComponent,
})

function RouteComponent() {
	const fen = Route.useLoaderData()

	return (
		<div className="h-full content-center">
			<Chessboard fen={fen} />
		</div>
	)
}
