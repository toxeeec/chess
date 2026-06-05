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
	const response = await server.requestHandler(async () => {
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

	return { result: result!, error, response }
}

export function redirect<TOptions>(options: ValidateRedirectOptions<RegisteredRouter, TOptions>) {
	return expect.objectContaining({ options: { statusCode: 307, ...options } })
}
