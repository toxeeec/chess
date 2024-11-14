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
				"relative inline-block rounded-full outline-none before:invisible before:absolute before:box-content before:size-full before:-translate-x-[12.5%] before:-translate-y-[12.5%] before:rounded-full before:bg-neutral-700 before:p-[12.5%] hover:before:visible focus-visible:before:visible [&>*]:relative",
			)}
		>
			{(renderProps) => (typeof children === "function" ? children(renderProps) : children)}
		</Link>
	)
}
