import { decrypt, encrypt } from "@/auth"
import { createUser, getUser } from "@/db/user"
import { NextRequest, NextResponse } from "next/server"

export type ErrorResponse = { error: string }
export type Token = { token: string }

export async function POST(request: NextRequest) {
	const key = request.headers.get("Authorization")?.split("Bearer ").at(1)
	if (!key || key !== process.env.INTERNAL_API_KEY) {
		return NextResponse.json<ErrorResponse>({ error: "invalid api key" }, { status: 401 })
	}

	let token = ((await request.json()) as Token).token
	if (token) {
		try {
			const { subject: userId } = await decrypt(token)
			const userExists = !!(await getUser(userId))
			if (userExists) {
				return NextResponse.json({ token })
			}
		} catch (e) {
			console.log(e)
		}
	}
	const user = await createUser()
	token = await encrypt(user.id)
	return NextResponse.json({ token })
}
