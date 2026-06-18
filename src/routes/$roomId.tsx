import { createFileRoute } from "@tanstack/react-router"
import { useRef, useState } from "react"

import { Board } from "#/board"
import { GameStoreContext, createGameStore } from "#/game-store"
import { PlayerClock } from "#/player-clock"
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

	const { send, reconnect } = useLiveRoom({
		roomId,
		onSnapshot: (snapshot) => {
			if (snapshot.revision <= lastRevision.current) return
			store.setState(snapshot)
			lastRevision.current = snapshot.revision
		},
		onStatus: ({ legalMoves, clock }) => {
			store.setState({ legalMoves, clock })
		},
		onMove: ({ revision, move, legalMoves, turn, clock }) => {
			if (revision <= lastRevision.current) return
			if (revision !== lastRevision.current + 1) {
				reconnect()
			} else {
				store.applyMove({ move, legalMoves, turn, clock })
				lastRevision.current = revision
			}
		},
	})

	const [store] = useState(() =>
		createGameStore({
			snapshot,
			player,
			onMove: (move) => {
				send({ type: "move", data: moveCodec.encode(move) })
			},
		}),
	)

	return (
		<div className="h-full content-center">
			<GameStoreContext value={store}>
				<div className="grid justify-center gap-[1.5vmin]">
					<PlayerClock player="black" />
					<Board onMove={store.movePiece} />
					<PlayerClock player="white" />
				</div>
			</GameStoreContext>
		</div>
	)
}
