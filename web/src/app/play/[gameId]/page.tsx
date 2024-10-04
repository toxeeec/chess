import { Chess } from "@/app/chess"

export default async function Game({ params }: { params: Promise<{ gameId: string }> }) {
	const { gameId } = await params

	return (
		<div className="grid h-full place-items-center">
			<div className="grid size-4/5 place-items-center">
				<Chess gameId={gameId} />
			</div>
		</div>
	)
}
