import { useRef } from "react"

export function useShallow<State, Selected extends readonly unknown[]>(
	selector: (state: State) => Selected,
) {
	const previous = useRef<Selected>(undefined)
	const hasPrevious = useRef(false)

	return (state: State) => {
		const next = selector(state)

		if (hasPrevious.current && shallow(previous.current!, next)) {
			return previous.current!
		}

		previous.current = next
		hasPrevious.current = true
		return next
	}
}

function shallow<T extends readonly unknown[]>(a: T, b: T) {
	if (Object.is(a, b)) return true
	if (a.length !== b.length) return false

	for (let i = 0; i < a.length; ++i) {
		if (!Object.is(a[i], b[i])) return false
	}
	return true
}
