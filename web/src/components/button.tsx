"use client"

import { composeRenderProps } from "@/utils"
import { ButtonProps, Button as RacButton } from "react-aria-components"

export function Button(props: ButtonProps) {
	return (
		<RacButton
			{...props}
			className={composeRenderProps(
				props.className,
				"rounded-lg bg-neutral-200 py-4 text-2xl font-semibold text-neutral-700 outline-none hover:bg-opacity-90 focus-visible:outline-neutral-200 pressed:bg-opacity-90 disabled:bg-opacity-50",
			)}
		/>
	)
}
