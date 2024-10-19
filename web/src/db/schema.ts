import { sqliteTable, text } from "drizzle-orm/sqlite-core"
import "server-only"

export const users = sqliteTable("users", {
	id: text().primaryKey(),
})

export type NewUser = typeof users.$inferInsert
