import { createGame } from "@/data/game"
import { redirect } from "next/navigation"

export default function GameLobby() {
	const gameId = createGame()
	redirect(`/play/${gameId}`)
}
