import type { RegisteredRouter, ValidateRedirectOptions } from "@tanstack/react-router"
import * as server from "@tanstack/react-start/server"
import { runWithStartContext } from "@tanstack/start-storage-context"
import { expect } from "vitest"

import { getRouter } from "./router"

export async function runTestServerFn<T>(
	fn: () => Promise<T>,
	requestInit?: RequestInit,
): Promise<T | Response> {
	const request = new Request("http://localhost/", requestInit)

	let result: T
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
			fn,
		)
		return new Response()
	})(request, {})

	if ("options" in response) return response
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
