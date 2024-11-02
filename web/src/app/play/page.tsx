import { Chess } from "../chess"

export default async function GameLobby() {
	return (
		<main className="flex h-full flex-col">
			<div className="grid flex-grow place-items-center px-6">
				<Chess />
			</div>
			<div className="rounded-t-2xl bg-neutral-800 p-6">bottom sheet</div>
		</main>
	)
}
