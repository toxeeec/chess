import * as schema from "./schema"
import Database from "better-sqlite3"
import { drizzle } from "drizzle-orm/better-sqlite3"

const sqlite = new Database("chess.db")
sqlite.pragma("busy_timeout = 5000")
sqlite.pragma("cache_size = 2000")
sqlite.pragma("foreign_keys = ON")
sqlite.pragma("journal_mode = WAL")
sqlite.pragma("synchronous = NORMAL")
sqlite.pragma("temp_store = memory")

export const db = drizzle(sqlite, { schema })
