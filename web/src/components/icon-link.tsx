"use client"

import { composeRenderProps } from "@/utils"
import { useRouter } from "next/navigation"
import { LinkProps, Link } from "react-aria-components"

export function IconLink({ href, children, ...props }: LinkProps) {
	const router = useRouter()
	return (
		<Link
			{...props}
			href={href}
			onHoverStart={() => href && router.prefetch(href)}
			onFocus={() => href && router.prefetch(href)}
			className={composeRenderProps(
				props.className,
				"inline-block rounded-full leading-none outline-none hover:opacity-90 focus-visible:outline-2 focus-visible:outline-neutral-200",
			)}
		>
			{(renderProps) => (typeof children === "function" ? children(renderProps) : children)}
		</Link>
	)
}
