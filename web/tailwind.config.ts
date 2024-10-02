import type { Config } from "tailwindcss"

const config: Config = {
	content: ["./src/app/**/*.{ts,tsx}"],
	theme: {
		extend: {
			borderWidth: {
				square: "var(--square-border-width)",
			},
			size: {
				board: "var(--board-size)",
			},
		},
	},
	plugins: [],
}
export default config
