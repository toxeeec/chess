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
			},
		},
	},
	plugins: [racPlugin],
}
export default config
