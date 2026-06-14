import { createFileRoute } from "@tanstack/react-router"
import { useRef, useState } from "react"

import { Board } from "#/board"
import { createBoardStore } from "#/board-store"
import { getGameState, ensureRoomSessionMatches } from "#/room"
import { moveCodec, snapshotMessageSchema, useLiveRoom } from "#/use-live-room"

import { parseRoomParams } from "./-room"

export const Route = createFileRoute("/$roomId")({
	params: {
		parse: parseRoomParams,
	},
	beforeLoad: ({ params }) => ensureRoomSessionMatches(params.roomId),
	loader: () => getGameState(),
	component: RouteComponent,
})

function RouteComponent() {
	const { player, ...data } = Route.useLoaderData()
	const { roomId } = Route.useParams()
	const snapshot = snapshotMessageSchema.parse(data)
	const lastRevision = useRef(-1)

	const [store] = useState(() =>
		createBoardStore({
			snapshot,
			player,
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
		onStatus: ({ legalMoves }) => {
			store.setLegalMoves(legalMoves)
		},
		onMove: ({ revision, move, legalMoves, turn }) => {
			if (revision <= lastRevision.current) return
			if (revision !== lastRevision.current + 1) {
				reconnect()
			} else {
				store.applyMove({ move, legalMoves, turn })
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
