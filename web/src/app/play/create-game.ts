"use server"

import { INCREMENT_VALUES, TIME_VALUES } from "./time-controls"
import { actionClient } from "@/safe-action"
import { createId } from "@paralleldrive/cuid2"
import { redirect } from "next/navigation"
import { z } from "zod"
import { zfd } from "zod-form-data"

const schema = zfd.formData({
	time: zfd.numeric(
		z
			.number()
			.min(0)
			.max(TIME_VALUES.length - 1),
	),
	increment: zfd.numeric(
		z
			.number()
			.min(0)
			.max(INCREMENT_VALUES.length - 1),
	),
})

export const createGame = actionClient.schema(schema).action(async () => {
	const id = createId()
	await fetch(`${process.env.GAME_SERVER_URL}/games`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify({ id }),
	})

	console.log(`created game ${id}`)
	redirect(`/play/${id}`)
})
