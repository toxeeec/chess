import { twMerge } from "@/utils"
import { LoaderCircle } from "lucide-react"
import { ProgressBar } from "react-aria-components"
import { ClassNameValue } from "tailwind-merge"

export function LoadingOverlay({
	isLoading,
	className,
	zIndex = 1,
}: {
	isLoading: boolean
	className?: ClassNameValue
	zIndex?: number
}) {
	return (
		<>
			<ProgressBar
				isIndeterminate
				aria-label="Loading..."
				className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"
				style={{ zIndex: zIndex + 1 }}
			>
				<LoaderCircle
					size={48}
					className={twMerge("animate-spin stroke-neutral-400", !isLoading && "hidden")}
				/>
			</ProgressBar>
			<div
				aria-hidden
				className={twMerge(
					"absolute inset-0 rounded-[inherit] backdrop-blur-[2px]",
					!isLoading && "hidden",
					className,
				)}
				style={{ zIndex }}
			></div>
		</>
	)
}
