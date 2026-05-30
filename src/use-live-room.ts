import { useRouter } from "@tanstack/react-router"
import { useEffect } from "react"
import { z } from "zod"

import type { RoomId } from "./room"
import { jsonCodec } from "./zod"

export const gameStateSchema = z.object({
	fen: z.string(),
	moves: z.string(),
})

export const liveRoomMessageCodec = jsonCodec(
	z.discriminatedUnion("type", [z.object({ type: z.literal("snapshot"), state: gameStateSchema })]),
)

export type GameState = z.infer<typeof gameStateSchema>
export type LiveRoomMessage = z.infer<typeof liveRoomMessageCodec>

export function useLiveRoom({
	roomId,
	onSnapshot,
}: {
	roomId: RoomId
	onSnapshot: (state: GameState, event: MessageEvent) => void
}) {
	const router = useRouter()

	useEffect(() => {
		const ws = new WebSocket(createLiveRoomUrl(roomId, router))

		function handleMessage(event: MessageEvent) {
			if (typeof event.data !== "string") {
				throw new TypeError("Live room websocket messages must be strings")
			}

			const message = liveRoomMessageCodec.parse(event.data)

			switch (message.type) {
				case "snapshot": {
					onSnapshot(message.state, event)
					return
				}
				default:
					message.type satisfies never
			}
		}

		ws.addEventListener("message", handleMessage)

		return () => {
			ws.removeEventListener("message", handleMessage)
			ws.close()
		}
	}, [onSnapshot, roomId, router])
}

function createLiveRoomUrl(roomId: RoomId, router: ReturnType<typeof useRouter>) {
	const wsUrl = new URL(
		router.buildLocation({ to: "/$roomId/ws", params: { roomId } }).publicHref,
		window.location.href,
	)
	wsUrl.protocol = wsUrl.protocol === "https:" ? "wss:" : "ws:"

	return wsUrl
}
