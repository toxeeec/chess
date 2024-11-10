"use client"

import { createGame } from "./create-game"
import { DEFAULT_INCREMENT, DEFAULT_TIME, INCREMENT_VALUES, TIME_VALUES } from "./time-controls"
import { Button } from "@/components/button"
import { LoadingOverlay } from "@/components/loading-overlay"
import { Slider } from "@/components/slider"
import { useAction } from "next-safe-action/hooks"
import Form from "next/form"

export function CreateGameForm() {
	const { execute, isPending } = useAction(createGame)

	return (
		<div className="relative rounded-t-[2rem] bg-neutral-800">
			<LoadingOverlay isLoading={isPending} className="bg-neutral-800/75" />
			<Form action={execute} className="flex flex-col gap-8 p-6">
				<Slider
					isDisabled={isPending}
					label="Time"
					name="time"
					defaultValue={TIME_VALUES.indexOf(DEFAULT_TIME)}
					maxValue={TIME_VALUES.length - 1}
					format={({ getThumbValue }) => {
						const value = TIME_VALUES[getThumbValue(0)]!
						return `${value} ${pluralize("minute", value)}`
					}}
				/>
				<Slider
					isDisabled={isPending}
					label="Increment"
					name="increment"
					defaultValue={INCREMENT_VALUES.indexOf(DEFAULT_INCREMENT)}
					maxValue={INCREMENT_VALUES.length - 1}
					format={({ getThumbValue }) => {
						const value = INCREMENT_VALUES[getThumbValue(0)]!
						return `${value} ${pluralize("second", value)}`
					}}
				/>
				<Button type="submit" className="mt-8" isDisabled={isPending}>
					Play
				</Button>
			</Form>
		</div>
	)
}

function pluralize(noun: string, count: number) {
	if (count === 1) return noun
	return `${noun}s`
}
