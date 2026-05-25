import { integer, sqliteTable, text } from "drizzle-orm/sqlite-core"

import type { RoomId } from "./room"

export const gamesTable = sqliteTable("games", {
	id: integer().primaryKey({ autoIncrement: true }),
	roomId: text().$type<RoomId>().unique().notNull(),
	white: text(),
	black: text(),
})
