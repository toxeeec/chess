"use client"

import { IconLink } from "@/components/icon-link"
import { Input } from "@/components/input"
import { ArrowLeft, Check, Clipboard } from "lucide-react"
import { usePathname } from "next/navigation"
import { useRef, useState } from "react"
import { Button, Label, TextField } from "react-aria-components"

export function PlayerInvite() {
	return (
		<div className="flex h-80 flex-col gap-6 rounded-t-[2rem] bg-neutral-800 p-6">
			<IconLink href="/play" className="self-start">
				<ArrowLeft size={32} className="stroke-neutral-200" />
			</IconLink>
			<div className="flex flex-grow flex-col justify-between">
				<TextField aria-label="Search player by name" className="contents">
					<Input placeholder="Search by name" />
				</TextField>
				<TextField className="flex flex-col gap-2">
					<Label className="font-semibold text-neutral-200">Copy game link</Label>
					<GameLink />
				</TextField>
			</div>
		</div>
	)
}

function GameLink() {
	const pathname = usePathname()
	const [copied, setCopied] = useState(false)

	const url = `${process.env.NEXT_PUBLIC_BASE_URL}${pathname}`
	const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

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
		<div className="flex">
			<Input
				value={url}
				readOnly
				variant="outline"
				className="flex-grow rounded-r-none border-r-0"
			/>
			<Button
				onPress={copyUrl}
				className="press:bg-neutral-600 grid size-10 place-items-center rounded-r-lg bg-neutral-700 text-neutral-400 outline-none hover:bg-neutral-600 focus-visible:border-2 focus-visible:border-neutral-500"
			>
				{copied ? <Check /> : <Clipboard />}
			</Button>
		</div>
	)
}
