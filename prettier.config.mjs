// @ts-check
/** @type {import("prettier").Config} */
const config = {
	plugins: ["@trivago/prettier-plugin-sort-imports", "prettier-plugin-tailwindcss"],
	printWidth: 100,
	tabWidth: 4,
	useTabs: true,
	semi: false,
}

export default config
