import type { Config } from "tailwindcss"

const config: Config = {
	content: ["./src/app/**/*.{ts,tsx}"],
	theme: {
		extend: {
			width: {
				board: "var(--board-width)",
			},
		},
	},
	plugins: [],
}
export default config
