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
	name?: string
}

export function Slider({ label, format, name, ...props }: SliderProps) {
	return (
		<RacSlider {...props}>
			<div className="flex justify-between pb-1 font-semibold text-neutral-200">
				<Label>{label}</Label>
				{format ? (
					<SliderOutput>{({ state }) => format(state)}</SliderOutput>
				) : (
					<SliderOutput />
				)}
			</div>
			<SliderTrack className="h-5">
				<div className="absolute top-1/2 h-2 w-full -translate-y-1/2 rounded-full bg-neutral-700"></div>
				<SliderThumb
					name={name}
					className="dragging:opacity-90 top-1/2 size-6 rounded-full border-2 border-neutral-800 bg-neutral-200 focus-visible:outline focus-visible:outline-neutral-200 disabled:bg-neutral-700"
				/>
			</SliderTrack>
		</RacSlider>
	)
}
