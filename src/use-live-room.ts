import { useRouter } from "@tanstack/react-router"
import { useEffect, useEffectEvent, useRef } from "react"
import { z } from "zod"

import type { RoomId } from "./room"
import { jsonCodec } from "./zod"

const MAX_RETRIES = 3
const BASE_DELAY_MS = 1_000

const squarePattern = "[a-h][1-8]"
const movePattern = `${squarePattern}${squarePattern}[qrbn]?`
const moveRegex = new RegExp(`^${movePattern}$`)
const movesRegex = new RegExp(`^(?:${movePattern}(?: ${movePattern})*)?$`)

const promotionPieceSchema = z.enum(["q", "r", "b", "n"])
type PromotionPiece = z.infer<typeof promotionPieceSchema>

const moveSchema = z.object({
	from: z.number(),
	to: z.number(),
	promotion: promotionPieceSchema.optional(),
})
export type Move = z.infer<typeof moveSchema>

function isPromotionPiece(value: string | undefined): value is PromotionPiece {
	return promotionPieceSchema.options.some((piece) => piece === value)
}

function decodeMove(moveStr: string) {
	const fromFile = moveStr[0]!.charCodeAt(0) - "a".charCodeAt(0)
	const fromRank = 8 - Number(moveStr[1])
	const toFile = moveStr[2]!.charCodeAt(0) - "a".charCodeAt(0)
	const toRank = 8 - Number(moveStr[3])
	const promotion = moveStr[4]
	return {
		from: fromRank * 8 + fromFile,
		to: toRank * 8 + toFile,
		...(isPromotionPiece(promotion) && { promotion }),
	}
}

function encodeMove(move: Move) {
	const fromFile = String.fromCharCode("a".charCodeAt(0) + (move.from % 8))
	const fromRank = 8 - Math.floor(move.from / 8)
	const toFile = String.fromCharCode("a".charCodeAt(0) + (move.to % 8))
	const toRank = 8 - Math.floor(move.to / 8)
	return `${fromFile}${fromRank}${toFile}${toRank}${move.promotion ?? ""}`
}

export const moveCodec = z.codec(z.string().regex(moveRegex, "Invalid move format"), moveSchema, {
	decode: decodeMove,
	encode: encodeMove,
})

const movesSchema = z
	.string()
	.regex(movesRegex, "Invalid moves format")
	.transform((moves) => (moves === "" ? [] : moves.split(" ").map(decodeMove)))

const playerSchema = z.enum(["white", "black"])

const clockSchema = z.object({
	whiteRemainingMs: z.number().int().nonnegative(),
	blackRemainingMs: z.number().int().nonnegative(),
	running: z.boolean(),
})

export type Clock = z.infer<typeof clockSchema>

export const snapshotMessageSchema = z.object({
	revision: z.number().int().nonnegative(),
	fen: z.string(),
	status: z.enum(["waiting", "active", "ended", "expired"]),
	clock: clockSchema,
	legalMoves: movesSchema,
})

const statusMessageSchema = z.object({
	status: snapshotMessageSchema.shape.status,
	clock: clockSchema,
	legalMoves: movesSchema,
})

const moveMessageSchema = z.object({
	revision: z.number().int().nonnegative(),
	move: moveCodec,
	turn: playerSchema,
	clock: clockSchema,
	legalMoves: movesSchema,
})

const liveRoomMessageCodec = jsonCodec(
	z.discriminatedUnion("type", [
		z.object({ type: z.literal("snapshot"), data: snapshotMessageSchema }),
		z.object({ type: z.literal("status"), data: statusMessageSchema }),
		z.object({ type: z.literal("move"), data: moveMessageSchema }),
		z.object({ type: z.literal("error"), data: z.string() }),
	]),
)

type SnapshotMessage = z.infer<typeof snapshotMessageSchema>
type StatusMessage = z.infer<typeof statusMessageSchema>
type MoveMessage = z.infer<typeof moveMessageSchema>

type ClientMessage = { type: "move"; data: string }

export function useLiveRoom({
	roomId,
	onSnapshot,
	onStatus,
	onMove,
}: {
	roomId: RoomId
	onSnapshot: (state: SnapshotMessage) => void
	onStatus: (status: StatusMessage) => void
	onMove: (state: MoveMessage) => void
}) {
	const router = useRouter()
	const ws = useRef<WebSocket | null>(null)
	const pendingMessages = useRef<ClientMessage[]>([])

	const handleMessage = useEffectEvent((event: MessageEvent) => {
		if (typeof event.data !== "string") {
			throw new Error("Live room websocket messages must be strings")
		}

		const message = liveRoomMessageCodec.parse(event.data)

		switch (message.type) {
			case "snapshot": {
				onSnapshot(message.data)
				return
			}
			case "status": {
				onStatus(message.data)
				return
			}
			case "move": {
				onMove(message.data)
				return
			}
			case "error": {
				throw new Error(message.data)
			}
			default: {
				message satisfies never
			}
		}
	})

	const send = (message: ClientMessage) => {
		if (ws.current?.readyState !== WebSocket.OPEN) {
			pendingMessages.current.push(message)
			return
		}

		ws.current.send(JSON.stringify(message))
	}

	const reconnect = () => {
		ws.current?.close()
	}

	const path = router.buildLocation({ to: "/$roomId/ws", params: { roomId } }).publicHref

	useEffect(() => {
		const url = createLiveRoomUrl(path)

		let disposed = false
		let reconnectTimer: ReturnType<typeof setTimeout> | undefined
		let reconnectRetryCount = 0

		const scheduleReconnect = () => {
			if (disposed) return

			if (reconnectRetryCount >= MAX_RETRIES) {
				throw new Error("Maximum reconnect retries exceeded")
			}
			const delay = BASE_DELAY_MS * 2 ** reconnectRetryCount
			reconnectRetryCount += 1
			reconnectTimer = setTimeout(() => {
				reconnectTimer = undefined
				if (!disposed) connect()
			}, delay)
		}

		const connect = () => {
			const socket = new WebSocket(url)
			ws.current = socket

			socket.addEventListener("open", () => {
				reconnectRetryCount = 0

				for (const message of pendingMessages.current) {
					socket.send(JSON.stringify(message))
				}
				pendingMessages.current = []
			})
			socket.addEventListener("message", handleMessage)
			socket.addEventListener("close", scheduleReconnect)
		}

		connect()

		return () => {
			disposed = true
			if (reconnectTimer) clearTimeout(reconnectTimer)
			pendingMessages.current = []
			ws.current?.close()
		}
	}, [path])

	return {
		send,
		reconnect,
	}
}

function createLiveRoomUrl(path: string) {
	const wsUrl = new URL(path, window.location.href)
	wsUrl.protocol = wsUrl.protocol === "https:" ? "wss:" : "ws:"

	return wsUrl
}

if (import.meta.vitest) {
	const { expect, it } = import.meta.vitest

	it.concurrent.each(["a1a1", "h8h8", "a7a8q", "a7a8r", "a7a8b", "a7a8n"])(
		"accepts valid move %s",
		(move) => {
			expect(() => moveCodec.decode(move)).not.toThrow()
		},
	)

	it.concurrent.each([
		"",
		"a1a1 ",
		" a1a1",
		"a1a1qextra",
		"a1a",
		"a1a11",
		"A1a1",
		"i1a1",
		"a0a1",
		"a1I1",
		"a1a9",
		"a7a8k",
	])("rejects invalid move %j", (move) => {
		expect(() => moveCodec.decode(move)).toThrow("Invalid move format")
	})

	it.concurrent.each(["", "a1a1", "a1a1 h8h8", "a7a8q h2h1n"])(
		"accepts valid move list %j",
		(moves) => {
			expect(() => movesSchema.parse(moves)).not.toThrow()
		},
	)

	it.concurrent.each([" ", "a1a1 ", " a1a1", "a1a1  h8h8", "a1a1\th8h8", "a1a1 h8h9"])(
		"rejects invalid move list %j",
		(moves) => {
			expect(() => movesSchema.parse(moves)).toThrow("Invalid moves format")
		},
	)

	it.concurrent("encodes and decodes moves", () => {
		expect(moveCodec.decode("a1h8")).toEqual({ from: 56, to: 7 })
		expect(moveCodec.decode("a7a8q")).toEqual({ from: 8, to: 0, promotion: "q" })
		expect(moveCodec.encode({ from: 55, to: 63, promotion: "n" })).toBe("h2h1n")
	})
}
