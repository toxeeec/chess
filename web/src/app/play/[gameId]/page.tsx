import { ChessGame } from "./chess-game"
import { PlayerInvite } from "./player-invite"
import { auth } from "@/auth"

export default async function Game({ params }: { params: Promise<{ gameId: string }> }) {
	const { gameId } = await params
	const { userId } = await auth()

	return (
		<main className="flex h-full flex-col">
			<div className="grid flex-grow place-items-center self-stretch">
				<ChessGame gameId={gameId} white={userId} />
			</div>
			<PlayerInvite />
		</main>
	)
}
