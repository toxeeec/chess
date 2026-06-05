import { createFileRoute } from "@tanstack/react-router"
import { useRef, useState } from "react"
import { z } from "zod"

import { Board } from "#/board"
import { createBoardStore } from "#/board-store"
import { getGameState, ensureRoomSessionMatches, roomIdSchema } from "#/room"
import { moveCodec, snapshotMessageSchema, useLiveRoom } from "#/use-live-room"

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
	const snapshot = snapshotMessageSchema.parse(Route.useLoaderData())
	const { roomId } = Route.useParams()
	const lastRevision = useRef(-1)

	const [store] = useState(() =>
		createBoardStore({
			snapshot,
			onMove: (move) => {
				send({ type: "move", data: moveCodec.encode(move) })
			},
		}),
	)
	const { send, reconnect } = useLiveRoom({
		roomId,
		onSnapshot: (snapshot) => {
			if (snapshot.revision <= lastRevision.current) return
			store.setState(snapshot)
			lastRevision.current = snapshot.revision
		},
		onMove: ({ revision, legalMoves }) => {
			if (revision <= lastRevision.current) return
			if (revision !== lastRevision.current + 1) {
				reconnect()
			} else {
				store.setLegalMoves(legalMoves)
				lastRevision.current = revision
			}
		},
	})

	return (
		<div className="h-full content-center">
			<Board store={store} />
		</div>
	)
}
