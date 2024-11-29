"use client"

import { IconLink } from "@/components/icon-link"
import { Input } from "@/components/input"
import { ArrowLeft, Check, Clipboard, Search } from "lucide-react"
import { usePathname } from "next/navigation"
import { useRef, useState } from "react"
import { Button, Label, TextField } from "react-aria-components"

export function PlayerInvite() {
	return (
		<div className="flex h-80 flex-col justify-between gap-6 rounded-t-[2rem] bg-neutral-800 p-6">
			<IconLink href="/play" className="self-start">
				<ArrowLeft size={32} className="stroke-neutral-200" />
			</IconLink>
			<h2 className="absolute left-1/2 -translate-x-1/2 text-xl font-semibold text-neutral-200">
				Challenge a player
			</h2>
			<div className="flex flex-col justify-between gap-6">
				<TextField aria-label="Search player by name" className="flex flex-col gap-2">
					<Label className="self-start font-semibold text-neutral-200">
						Search by name
					</Label>
					<div className="relative">
						<Input className="w-full pl-9" />
						<Search
							className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 stroke-neutral-500 stroke-[3]"
							size={16}
						/>
					</div>
				</TextField>
				<TextField className="flex flex-col gap-2">
					<Label className="self-start font-semibold text-neutral-200">
						Copy game link
					</Label>
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
