"use client"

import { IconLink } from "@/components/icon-link"
import { ArrowLeft, Check, Clipboard } from "lucide-react"
import { usePathname } from "next/navigation"
import { useRef, useState } from "react"
import { Button, Input } from "react-aria-components"

export function PlayerInvite() {
	return (
		<div className="flex h-80 flex-col rounded-t-[2rem] bg-neutral-800 p-6">
			<IconLink href="/play" className="self-start">
				<ArrowLeft size={32} className="stroke-neutral-200" />
			</IconLink>
			<div className="flex flex-grow flex-col justify-end gap-2">
				<h2 className="font-semibold text-neutral-200">Copy game link</h2>
				<GameLink />
			</div>
		</div>
	)
}

function GameLink() {
	const pathname = usePathname()
	const [copied, setCopied] = useState(false)

	const url = `${process.env.NEXT_PUBLIC_BASE_URL}${pathname}`
	const timeoutRef = useRef<Timer | null>(null)
	const copyUrl = () => {
		navigator.clipboard.writeText(url)

		setCopied(true)
		if (timeoutRef.current) clearTimeout(timeoutRef.current)
		timeoutRef.current = setTimeout(() => {
			setCopied(false)
			timeoutRef.current = null
		}, 1000)
	}

	return (
		<div className="flex h-10 text-neutral-400">
			<Input
				value={url}
				readOnly
				className="h-full flex-grow text-ellipsis rounded-l-lg rounded-r-none border border-r-0 border-neutral-700 bg-transparent px-3 outline-none"
			/>
			<Button
				onPress={copyUrl}
				className="press:bg-neutral-600 grid size-10 place-items-center rounded-r-lg bg-neutral-700 outline-none hover:bg-neutral-600 focus-visible:border-2 focus-visible:border-neutral-500"
			>
				{copied ? <Check /> : <Clipboard />}
			</Button>
		</div>
	)
}
