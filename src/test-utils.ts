import type { AnyRedirect, RegisteredRouter, ValidateRedirectOptions } from "@tanstack/react-router"
import { isRedirect } from "@tanstack/react-router"
import * as server from "@tanstack/react-start/server"
import { runWithStartContext } from "@tanstack/start-storage-context"
import { expect } from "vitest"

import { getRouter } from "./router"

export async function runInStartContext<T>(fn: () => T | Promise<T>, requestInit?: RequestInit) {
	const request = new Request("https://chess.localhost", requestInit)

	let result: AnyRedirect | Awaited<T> | undefined
	let error: unknown
	await server.requestHandler(async () => {
		result = await runWithStartContext(
			{
				getRouter,
				request,
				startOptions: {},
				contextAfterGlobalMiddlewares: {},
				executedRequestMiddlewares: new Set(),
				handlerType: "serverFn",
			},
			async () => {
				try {
					return await fn()
				} catch (err) {
					if (isRedirect(err)) return err
					error = err
					return undefined
				}
			},
		)
		return new Response()
	})(request, {})

	if (error) throw error
	return result!
}

export async function expectRedirect<T, TOptions>(
	promise: T | Promise<T>,
	options: ValidateRedirectOptions<RegisteredRouter, TOptions>,
) {
	await expect(Promise.try(() => promise)).resolves.toEqual(
		expect.objectContaining({ options: { statusCode: 307, ...options } }),
	)
}
