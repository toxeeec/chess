import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/")({ component: Home })

function Home() {
	return (
		<div className="h-full content-center">
			<Chessboard />
		</div>
	)
}

function Chessboard() {
	return (
		<div className="grid size-[round(down,80vmin,8px)] grid-cols-8 justify-self-center">
			{Array.from({ length: 64 }, (_, index) => index).map((square) => {
				const row = Math.floor(square / 8)
				const file = square % 8
				const isLight = (row + file) % 2 === 0
				return <div key={square} className={isLight ? "bg-neutral-300" : "bg-neutral-800"} />
			})}
		</div>
	)
}
