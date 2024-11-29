"use client"

import { twMerge } from "@/utils"
import { useRouter } from "next/navigation"
import { LinkProps, Link, composeRenderProps } from "react-aria-components"

export function IconLink(props: LinkProps) {
	const router = useRouter()
	return (
		<Link
			{...props}
			onHoverStart={() => props.href && router.prefetch(props.href)}
			onFocus={() => props.href && router.prefetch(props.href)}
			className={composeRenderProps(props.className, (className) =>
				twMerge(
					"relative inline-block rounded-full outline-none before:invisible before:absolute before:size-[125%] before:-translate-x-[10%] before:-translate-y-[10%] before:rounded-full before:bg-neutral-700 before:p-[12.5%] hover:before:visible focus-visible:before:visible [&>*]:relative",
					className,
				),
			)}
		/>
	)
}
