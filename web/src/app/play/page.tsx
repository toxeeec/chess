import { Chess } from "../chess"
import { CreateGameForm } from "./create-game-form"

export default async function GameLobby() {
	return (
		<main className="flex h-full flex-col">
			<div className="grid flex-grow place-items-center px-6">
				<Chess />
			</div>
			<CreateGameForm />
		</main>
	)
}
