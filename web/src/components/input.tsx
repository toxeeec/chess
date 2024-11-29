"use client"

import {
	InputProps as RacInputProps,
	Input as RacInput,
	composeRenderProps,
} from "react-aria-components"
import { tv, VariantProps } from "tailwind-variants"

const input = tv({
	base: "h-10 text-ellipsis rounded-lg px-3 caret-current outline-none",
	variants: {
		variant: {
			primary:
				"border-2 border-transparent bg-neutral-700 text-neutral-200 focus:border-neutral-500",
			outline: "border border-neutral-700 bg-transparent text-neutral-400",
		},
	},
	defaultVariants: { variant: "primary" },
})

interface InputProps extends RacInputProps, VariantProps<typeof input> {}

export function Input({ variant, ...props }: InputProps) {
	return (
		<RacInput
			{...props}
			className={composeRenderProps(props.className, (className) =>
				input({ variant, className }),
			)}
		/>
	)
}
