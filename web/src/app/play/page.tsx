import { createGame } from "@/data/game"
import { redirect } from "next/navigation"

export default async function GameLobby() {
	const gameId = await createGame()
	redirect(`/play/${gameId}`)
}
