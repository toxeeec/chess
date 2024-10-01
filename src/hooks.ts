import { RefObject, useEffect } from "react"

export function useResizeObserver(
	ref: RefObject<HTMLElement>,
	callback: (entry: ResizeObserverEntry) => void,
) {
	useEffect(() => {
		const observer = new ResizeObserver(([entry]) => {
			if (entry) callback(entry)
		})
		if (ref.current) observer.observe(ref.current)
		return () => observer.disconnect()
	}, [ref, callback])
}
