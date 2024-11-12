import type { Config } from "tailwindcss"
import racPlugin from "tailwindcss-react-aria-components"

const config: Config = {
	content: ["./src/**/*.{ts,tsx}"],
	theme: {
		extend: {
			borderWidth: {
				square: "var(--square-border-width)",
			},
			size: {
				board: "var(--board-size)",
				"board-container": "calc((100% * 8/9) - 16px)",
			},
		},
	},
	plugins: [racPlugin],
}
export default config
