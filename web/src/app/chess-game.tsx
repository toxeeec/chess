"use client"

import { Chess } from "./chess"
import { useRouter } from "next/navigation"
import { useEffect } from "react"
import { io } from "socket.io-client"

export function ChessGame({ gameId }: { gameId: string }) {
	const router = useRouter()

	useEffect(() => {
		const socket = io(process.env.NEXT_PUBLIC_GAME_SERVER_URL)

		socket.on("connect", () => {
			socket.emit("join", gameId)
		})

		socket.on("message", (e) => {
			console.log(e)
		})

		socket.on("disconnect", () => {
			router.replace("/play")
		})
	}, [gameId, router])

	return <Chess />
}
