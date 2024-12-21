import { Chess } from "./chess"
import { CreateGameForm } from "./create-game-form"
import { auth } from "@/auth"

export default async function GameLobby() {
	const { userId } = await auth()

	return (
		<main className="flex h-full flex-col">
			<div className="grid flex-grow place-items-center self-stretch">
				<Chess white={userId} black="Opponent" />
			</div>
			<CreateGameForm />
		</main>
	)
}
