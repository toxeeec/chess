import { db } from "./drizzle"
import { NewUser, users } from "./schema"
import { createId } from "@paralleldrive/cuid2"
import { eq } from "drizzle-orm"

export async function createUser() {
	const userId = createId()
	const user = { id: userId } satisfies NewUser
	await db.insert(users).values(user)
	return user
}

export function getUser(userId: string) {
	return db.query.users.findFirst({ where: eq(users.id, userId) })
}
