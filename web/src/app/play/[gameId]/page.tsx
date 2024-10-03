import { Chess } from "@/app/chess"
import { joinGame } from "@/data/game"

export default async function Game({ params }: { params: Promise<{ gameId: string }> }) {
	const { gameId } = await params
	await joinGame(gameId)

	return (
		<div className="grid h-full place-items-center">
			<div className="grid size-4/5 place-items-center">
				<Chess />
			</div>
		</div>
	)
}
