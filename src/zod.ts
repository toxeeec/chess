import { z } from "zod"

export const jsonCodec = <T extends z.core.$ZodType>(schema: T) =>
	z.codec(z.string(), schema, {
		decode: (jsonString, ctx) => {
			try {
				// oxlint-disable-next-line
				return JSON.parse(jsonString) as any
			} catch (err) {
				ctx.issues.push({
					code: "invalid_format",
					format: "json",
					input: jsonString,
					message: String(err),
				})
				return z.NEVER
			}
		},
		encode: (value) => JSON.stringify(value),
	})
