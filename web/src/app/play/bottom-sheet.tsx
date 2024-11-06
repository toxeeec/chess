"use client"

import { Button } from "@/components/button"
import { Slider } from "@/components/slider"

const TIME_VALUES = [1, 2, 3, 5, 10, 15, 30] as const
const INCREMENT_VALUES = [0, 1, 2, 3, 5, 10, 20] as const

const DEFAULT_TIME = 10 satisfies (typeof TIME_VALUES)[number]
const DEFAULT_INCREMENT = 0 satisfies (typeof INCREMENT_VALUES)[number]

export function BottomSheet() {
	return (
		<div className="flex flex-col gap-8 rounded-t-[2rem] bg-neutral-800 p-6">
			<Slider
				label="Time"
				defaultValue={TIME_VALUES.indexOf(DEFAULT_TIME)}
				maxValue={TIME_VALUES.length - 1}
				format={({ getThumbValue }) => {
					const value = TIME_VALUES[getThumbValue(0)]!
					return `${value} ${pluralize("minute", value)}`
				}}
			/>
			<Slider
				label="Increment"
				defaultValue={INCREMENT_VALUES.indexOf(DEFAULT_INCREMENT)}
				maxValue={INCREMENT_VALUES.length - 1}
				format={({ getThumbValue }) => {
					const value = INCREMENT_VALUES[getThumbValue(0)]!
					return `${value} ${pluralize("second", value)}`
				}}
			/>
			<Button className="mt-8">Play</Button>
		</div>
	)
}

function pluralize(noun: string, count: number) {
	if (count === 1) return noun
	return `${noun}s`
}
