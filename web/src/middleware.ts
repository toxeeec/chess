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
			return NextResponse.redirect(
				new URL(`/error?status=${res.status}&message=${error}`, request.url),
			)
		}
		const newToken = ((await res.json()) as Token).token
		const response = NextResponse.next()
		setToken(response, newToken)
		return response
	} catch (e) {
		let url = "/error"
		if (e instanceof Error) {
			url += `?message=${e.message}`
		}
		return NextResponse.redirect(new URL(url, request.url))
	}
}

export const config = { matcher: "/((?!api|_next|error|.*\\..*).*)" }
