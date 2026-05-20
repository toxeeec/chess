import { createFileRoute } from "@tanstack/react-router"

import { clsx } from "#/clsx"

export const Route = createFileRoute("/")({ component: Home })

const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"] as const
const RANKS = [8, 7, 6, 5, 4, 3, 2, 1] as const

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

				return (
					<div
						key={square}
						className={clsx("relative", isLight ? "bg-neutral-300" : "bg-neutral-800")}
					>
						{file === 0 && (
							<span
								className={clsx(
									"absolute top-[6%] left-[6%] text-[2.2vmin] leading-none font-semibold select-none",
									isLight ? "text-neutral-800" : "text-neutral-300",
								)}
							>
								{RANKS[row]}
							</span>
						)}
						{row === 7 && (
							<span
								className={clsx(
									"absolute right-[6%] bottom-[6%] text-[2.2vmin] leading-none font-semibold select-none",
									isLight ? "text-neutral-800" : "text-neutral-300",
								)}
							>
								{FILES[file]}
							</span>
						)}
					</div>
				)
			})}
		</div>
	)
}
