import { ChessGame } from "@/app/play/[gameId]/chess-game"

export default async function Game({ params }: { params: Promise<{ gameId: string }> }) {
	const { gameId } = await params

	return (
		<div className="grid h-full place-items-center">
			<div className="grid size-4/5 place-items-center">
				<ChessGame gameId={gameId} />
			</div>
		</div>
	)
}
