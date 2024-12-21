import type { Config } from "tailwindcss"
import racPlugin from "tailwindcss-react-aria-components"

const config: Config = {
	content: ["./src/**/*.{ts,tsx}"],
	theme: {
		extend: {
			borderWidth: {
				square: "var(--square-border-width)",
			},
			width: {
				board: "var(--board-size, 0)",
			},
			size: {
				board: "var(--board-size, 0)",
				"board-container": "var(--board-container-size)",
			},
		},
	},
	plugins: [racPlugin],
}
export default config
