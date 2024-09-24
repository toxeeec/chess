import { extendTailwindMerge } from "tailwind-merge"

export const twMerge = extendTailwindMerge({
	extend: {
		classGroups: {
			w: [{ w: ["board"] }],
		},
	},
})
