const FILES = ["A", "B", "C", "D", "E", "F", "G", "H"] as const
const RANKS = [1, 2, 3, 4, 5, 6, 7, 8] as const
export const SQUARES = FILES.flatMap((file) => RANKS.map((rank) => `${file}${rank}` as const))

export type Square = (typeof SQUARES)[number]

export namespace Square {
	export function fromIndex(index: number) {
		const file = index % 8
		const rank = Math.floor(index / 8)

		return `${FILES[file]}${RANKS[rank]}` as Square
	}

	export function file(square: Square) {
		return square.charCodeAt(0) - "A".charCodeAt(0)
	}

	export function rank(square: Square) {
		return 8 - parseInt(square[1] as string)
	}

	export function gridArea(square: Square) {
		return `${rank(square) + 1} / ${file(square) + 1}`
	}

	export function isLight(square: Square) {
		return file(square) % 2 === rank(square) % 2
	}
}
