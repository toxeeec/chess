import { createId } from "@paralleldrive/cuid2"
import { redirect } from "next/navigation"
import "server-only"

const GAMES = new Set<string>()

export function createGame() {
	const id = createId()
	GAMES.add(id)
	console.log(`created game ${id}`)
	return id
}

export function joinGame(gameId: string) {
	if (!GAMES.has(gameId)) {
		console.log(`game ${gameId} not found`)
		const id = createGame()
		redirect(`/play/${id}`)
	}
	console.log(`joined game ${gameId}`)
}
