import { ErrorResponse, Token } from "./app/api/users/route"
import { getToken, setToken } from "./auth"
import { NextRequest, NextResponse } from "next/server"

export async function middleware(request: NextRequest) {
	const token = getToken(request)

	try {
		const res = await fetch(`${request.nextUrl.origin}/api/users`, {
			method: "POST",
			headers: {
				Authorization: `Bearer ${process.env.INTERNAL_API_KEY}`,
			},
			body: JSON.stringify({
				token,
			}),
		})
		if (!res.ok) {
			const { error } = (await res.json()) as ErrorResponse
			console.log(error)
			// TODO: add error page
		}
		const newToken = ((await res.json()) as Token).token
		const response = NextResponse.next()
		setToken(response, newToken)
		return response
	} catch (e) {
		console.log(e)
		// TODO: add error page
	}
}

export const config = { matcher: "/((?!api|_next|error|.*\\..*).*)" }
