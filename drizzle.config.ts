import { defineConfig } from "drizzle-kit"

export default defineConfig({
	dialect: "sqlite",
	schema: "./src/schema.server.ts",
	driver: "d1-http",
	dbCredentials: {
		databaseId: "6f626adc-c7c8-4dbd-a9fb-7bed0ac2a299",
		accountId: process.env.CLOUDFLARE_ACCOUNT_ID,
		token: process.env.CLOUDFLARE_D1_API_TOKEN,
	},
	verbose: true,
	strict: true,
})
