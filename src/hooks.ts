import { useState } from "react"

export function useElementSize() {
	const [dimensions, setDimensions] = useState({
		width: 0,
		height: 0,
	})

	const ref = (el: HTMLElement | null) => {
		if (el?.nodeType !== Node.ELEMENT_NODE) return

		const observer = new ResizeObserver(([entry]) => {
			if (entry?.borderBoxSize?.[0]) {
				const { inlineSize: width, blockSize: height } = entry.borderBoxSize[0]
				setDimensions({ width, height })
			}
		})
		observer.observe(el)

		return () => {
			observer.unobserve(el)
		}
	}

	return { ref, dimensions }
}
