import { jwtVerify, SignJWT } from "jose"
import { cookies } from "next/headers"
import { NextRequest, NextResponse } from "next/server"
import { cache } from "react"
import "server-only"

const ALG = "HS256"
const SECRET = new TextEncoder().encode(process.env.AUTH_SECRET)
const ISSUER = "chess"

const TOKEN_COOKIE_NAME = "token"
const COOKIE_MAX_AGE = 30 * 24 * 60 * 60 * 1000 // 30 days

export async function encrypt(subject: string) {
	return new SignJWT()
		.setIssuer(ISSUER)
		.setSubject(subject)
		.setIssuedAt()
		.setProtectedHeader({ alg: ALG })
		.sign(SECRET)
}

export async function decrypt(jwt: string) {
	const { payload } = await jwtVerify(jwt, SECRET, {
		algorithms: [ALG],
		issuer: ISSUER,
	})
	return { subject: payload.sub! }
}

export function getToken(request: NextRequest) {
	return request.cookies.get(TOKEN_COOKIE_NAME)?.value
}

export function setToken(response: NextResponse, token: string) {
	const expires = new Date().getTime() + COOKIE_MAX_AGE
	response.cookies.set({
		name: TOKEN_COOKIE_NAME,
		value: token,
		expires,
		httpOnly: true,
	})
}

export const auth = cache(async () => {
	const token = (await cookies()).get(TOKEN_COOKIE_NAME)?.value
	if (!token) {
		throw new Error("token not found")
	}
	const { subject: userId } = await decrypt(token)
	return { userId }
})
