import { z } from "zod"

import { roomIdSchema } from "#/room"

const roomParamsSchema = z.object({ roomId: roomIdSchema })

export function parseRoomParams(params: unknown) {
	const { data, success } = roomParamsSchema.safeParse(params)
	if (!success) return false
	return data
}
