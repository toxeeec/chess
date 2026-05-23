import { createFileRoute } from "@tanstack/react-router"

import { Chessboard } from "#/chessboard"
export const Route = createFileRoute("/")({ component: Home })
function Home() {
	return (
		<div className="h-full content-center">
			<Chessboard />
		</div>
	)
}
