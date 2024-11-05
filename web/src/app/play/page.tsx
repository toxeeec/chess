import { Chess } from "../chess"
import { Button } from "@/components/button"

export default async function GameLobby() {
	return (
		<main className="flex h-full flex-col">
			<div className="grid flex-grow place-items-center px-6">
				<Chess />
			</div>
			<div className="flex flex-col rounded-t-[2rem] bg-neutral-800 p-6">
				<Button className="mt-40">Play</Button>
			</div>
		</main>
	)
}
