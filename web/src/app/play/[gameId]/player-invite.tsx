"use client"

import { IconLink } from "@/components/icon-link"
import { ArrowLeft } from "lucide-react"

export function PlayerInvite() {
	return (
		<div className="rounded-t-[2rem] bg-neutral-800 p-6">
			<IconLink href="/play">
				<ArrowLeft size={32} className="stroke-neutral-200" />
			</IconLink>
		</div>
	)
}
