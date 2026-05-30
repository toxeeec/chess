import { createFileRoute } from "@tanstack/react-router"
import { useState } from "react"
import { z } from "zod"

import { createBoardStore } from "#/board-store"
import { Chessboard } from "#/chessboard"
import { getGameState, ensureRoomSessionMatches, roomIdSchema } from "#/room"
import { gameStateSchema, useLiveRoom } from "#/use-live-room"

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
	loader: () => getGameState(),
	component: RouteComponent,
})

function RouteComponent() {
	const state = gameStateSchema.parse(Route.useLoaderData())
	const { roomId } = Route.useParams()

	const [store] = useState(() => createBoardStore(state))
	useLiveRoom({ roomId, onSnapshot: store.setState })

	return (
		<div className="h-full content-center">
			<Chessboard store={store} />
		</div>
	)
}
