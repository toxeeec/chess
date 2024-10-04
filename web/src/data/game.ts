import { GAME_SERVER_URL } from "@/env"
import { createId } from "@paralleldrive/cuid2"
import "server-only"

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
