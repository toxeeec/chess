import { createId } from "@paralleldrive/cuid2"
import { redirect } from "next/navigation"
import "server-only"

const GAME_SERVER_URL = "http://localhost:3001"

export async function createGame() {
	const id = createId()
	await fetch(`${GAME_SERVER_URL}/games`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		cache: "no-store",
		body: JSON.stringify({ id }),
	})
	console.log(`created game ${id}`)
	return id
}

export async function joinGame(id: string) {
	const res = await fetch(`${GAME_SERVER_URL}/games/${id}`, { cache: "no-store" })
	if (res.status === 404) {
		console.log(`game ${id} not found`)
		const newId = await createGame()
		redirect(`/play/${newId}`)
	}
	console.log(`joined game ${id}`)
}
