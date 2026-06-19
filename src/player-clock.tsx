import { useEffect, useState } from "react"

import { clsx } from "./clsx"
import { useGameStore, useShallow } from "./game-store"
import type { Player } from "./room"

export function PlayerClock({ player }: { player: Player }) {
	const [active, remainingMs, receivedAtMs] = useGameStore(
		useShallow((store) => [
			store.clock.running && store.turn === player,
			player === "white" ? store.clock.whiteRemainingMs : store.clock.blackRemainingMs,
			store.clock.receivedAtMs,
		]),
	)

	if (active) {
		return <ActiveClock player={player} remainingMs={remainingMs} receivedAtMs={receivedAtMs} />
	}
	return <ClockDisplay active={false} player={player} remainingMs={remainingMs} />
}

function ActiveClock({
	player,
	remainingMs,
	receivedAtMs,
}: {
	player: Player
	remainingMs: number
	receivedAtMs: number
}) {
	const [now, setNow] = useState(() => Date.now())

	useEffect(() => {
		const interval = setInterval(() => setNow(Date.now()), 250)
		return () => clearInterval(interval)
	}, [])

	const liveRemainingMs = Math.max(0, remainingMs - (now - receivedAtMs))
	return <ClockDisplay active player={player} remainingMs={liveRemainingMs} />
}

function ClockDisplay({
	active,
	player,
	remainingMs,
}: {
	active: boolean
	player: Player
	remainingMs: number
}) {
	return (
		<div
			className={clsx(
				"justify-self-end px-[1.5vmin] py-[0.75vmin] text-[3.5vmin] leading-none font-semibold tabular-nums",
				player === "white" &&
					(active ? "bg-neutral-100 text-neutral-900" : "bg-neutral-500 text-neutral-900"),
				player === "black" &&
					(active ? "bg-neutral-900 text-neutral-100" : "bg-neutral-900 text-neutral-500"),
			)}
		>
			{formatClock(remainingMs)}
		</div>
	)
}

function formatClock(ms: number) {
	const totalSeconds = Math.ceil(ms / 1000)
	const minutes = Math.floor(totalSeconds / 60)
	const seconds = totalSeconds % 60

	return `${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`
}
