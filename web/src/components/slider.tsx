"use client"

import {
	SliderProps as RacSliderProps,
	Slider as RacSlider,
	Label,
	SliderOutput,
	SliderTrack,
	SliderThumb,
} from "react-aria-components"
import { SliderState } from "react-stately"

interface SliderProps extends Omit<RacSliderProps, "className"> {
	label: string
	format?: (state: SliderState) => string
}

export function Slider({ label, format, ...props }: SliderProps) {
	return (
		<RacSlider {...props}>
			<div className="flex pb-1 font-semibold text-neutral-200">
				<Label className="flex-1">{label}</Label>
				{format ? (
					<SliderOutput>{({ state }) => format(state)}</SliderOutput>
				) : (
					<SliderOutput />
				)}
			</div>
			<SliderTrack className="h-5">
				<div className="absolute top-1/2 h-2 w-full -translate-y-1/2 rounded-full bg-neutral-700"></div>
				<SliderThumb className="dragging:opacity-90 top-1/2 h-5 w-5 rounded-full bg-neutral-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-neutral-200" />
			</SliderTrack>
		</RacSlider>
	)
}
