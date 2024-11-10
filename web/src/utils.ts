import { Modifier } from "@dnd-kit/core"
import { composeRenderProps as racComposeRenderProps } from "react-aria-components"
import { extendTailwindMerge } from "tailwind-merge"

export const restrictToParentElement: Modifier = ({
	containerNodeRect,
	draggingNodeRect,
	transform,
}) => {
	if (!draggingNodeRect || !containerNodeRect) return transform

	const halfWidth = draggingNodeRect.width / 2
	const halfHeight = draggingNodeRect.height / 2

	const minX = containerNodeRect.left - draggingNodeRect.left - halfWidth
	const maxX = containerNodeRect.right - draggingNodeRect.right + halfWidth
	const minY = containerNodeRect.top - draggingNodeRect.top - halfHeight
	const maxY = containerNodeRect.bottom - draggingNodeRect.bottom + halfHeight

	transform.x = clamp(transform.x, minX, maxX)
	transform.y = clamp(transform.y, minY, maxY)

	return transform
}

export function composeRenderProps<T extends Parameters<typeof twMerge>[number], U, V extends T>(
	value: Parameters<typeof racComposeRenderProps<T, U, V>>[0],
	...classLists: T[]
) {
	return racComposeRenderProps(value, (prevValue) => twMerge(classLists, prevValue) as V)
}

export const twMerge = extendTailwindMerge({
	extend: {
		theme: {
			borderWidth: ["square"],
		},
		classGroups: {
			size: [{ size: ["board"] }],
		},
	},
})

function clamp(val: number, min: number, max: number) {
	return Math.min(Math.max(val, min), max)
}
