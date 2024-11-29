import { ChessGame } from "./chess-game"
import { PlayerInvite } from "./player-invite"

export default async function Game({ params }: { params: Promise<{ gameId: string }> }) {
	const { gameId } = await params

	return (
		<main className="flex h-full flex-col">
			<div className="grid flex-grow place-items-center self-stretch">
				<div className="grid size-board-container place-items-center">
					<ChessGame gameId={gameId} />
				</div>
			</div>
			<PlayerInvite />
		</main>
	)
}
