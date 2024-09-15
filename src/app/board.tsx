import { ReactNode } from "react"

export function Board({ children }: { children?: ReactNode }) {
	return (
		<div className="w-board relative grid aspect-square grid-cols-8 grid-rows-8">
			{children}
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 8 8"
				className="absolute inset-0 -z-10"
			>
				<path className="fill-neutral-700" d="M0 0h8v8H0" />
				<path
					className="fill-neutral-400"
					d="M0 0h8v1H0m0 1h8v1H0m0 1h8v1H0m0 1h8v1H0m1-7v8h1V0m1 0v8h1V0m1 0v8h1V0m1 0v8h1V0"
				/>
			</svg>
		</div>
	)
}
