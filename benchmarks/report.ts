import { spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs"
import { tmpdir } from "node:os"
import { join } from "node:path"

import { z } from "zod"

export type BenchmarkTarget = "native" | "wasm"

const throughputFormatter = new Intl.NumberFormat("en-US", {
	compactDisplay: "short",
	maximumFractionDigits: 2,
	minimumFractionDigits: 2,
	notation: "compact",
})

const estimateSchema = z.object({
	confidence_interval: z.object({
		lower_bound: z.number(),
		upper_bound: z.number(),
	}),
	point_estimate: z.number(),
})

const criterionBenchmarkSchema = z.object({
	full_id: z.string().min(1),
	throughput: z.object({ Elements: z.number().nonnegative() }),
})

const criterionEstimatesSchema = z.object({
	mean: estimateSchema,
	slope: estimateSchema.nullable(),
})

const criterionSampleSchema = z.object({ iters: z.array(z.number().positive()).min(1) })

const vitestBenchmarkSchema = z.object({
	hz: z.number().positive(),
	name: z.string().min(1),
	rme: z.number().nonnegative(),
	sampleCount: z.number().int().positive(),
})

const vitestOutputSchema = z.object({
	files: z.array(
		z.object({
			groups: z.array(
				z.object({
					benchmarks: z.array(vitestBenchmarkSchema),
				}),
			),
		}),
	),
})

type RunBenchmarkOptions = { target: BenchmarkTarget } & (
	| {
			benchmark: "games"
			datasetId: string
			datasetPath: string
	  }
	| {
			benchmark: "perft"
			cases: { depth: number; fen: string }[]
			label: string
	  }
)
type BenchmarkName = RunBenchmarkOptions["benchmark"]

export function runBenchmark(options: RunBenchmarkOptions) {
	const { benchmarkId, environment } = benchmarkConfiguration(options)
	const unit = options.benchmark === "perft" ? "nodes" : "moves"
	Object.assign(process.env, environment)

	switch (options.target) {
		case "native": {
			return runNativeBenchmark(options.benchmark, benchmarkId, unit)
		}
		case "wasm": {
			return runWasmBenchmark(options.benchmark, unit)
		}
		default: {
			return options.target satisfies never
		}
	}
}

function benchmarkConfiguration(options: RunBenchmarkOptions) {
	switch (options.benchmark) {
		case "games": {
			return {
				benchmarkId: `games/${options.datasetId}`,
				environment: {
					GAME_DATASET_ID: options.datasetId,
					GAME_DATASET_PATH: options.datasetPath,
				},
			}
		}
		case "perft": {
			return {
				benchmarkId: `perft/${options.label}`,
				environment: {
					PERFT_CASES: JSON.stringify(options.cases),
					PERFT_LABEL: options.label,
				},
			}
		}
		default: {
			return options satisfies never
		}
	}
}

function runNativeBenchmark(
	benchmarkName: BenchmarkName,
	benchmarkId: string,
	unit: "moves" | "nodes",
) {
	const outputDirectory = mkdtempSync(join(tmpdir(), "chess-criterion-"))

	try {
		run(
			"cargo",
			[
				"bench",
				"--manifest-path",
				"game-server/Cargo.toml",
				"--bench",
				benchmarkName,
				"--features",
				"benchmark",
				"--profile",
				"benchmark-native",
			],
			{
				...process.env,
				CRITERION_HOME: outputDirectory,
				RUSTFLAGS: `${process.env.RUSTFLAGS ?? ""} -Ctarget-cpu=native`.trim(),
			},
		)

		const resultDirectory = findCriterionResult(outputDirectory, benchmarkId)
		const benchmarkResult = criterionBenchmarkSchema.parse(
			readJson(join(resultDirectory, "benchmark.json")),
		)
		const estimates = criterionEstimatesSchema.parse(
			readJson(join(resultDirectory, "estimates.json")),
		)
		const estimate = estimates.slope ?? estimates.mean
		const sample = criterionSampleSchema.parse(readJson(join(resultDirectory, "sample.json")))
		const elementsPerIteration = benchmarkResult.throughput.Elements

		const throughput = {
			lower: (elementsPerIteration * 1e9) / estimate.confidence_interval.upper_bound,
			point: (elementsPerIteration * 1e9) / estimate.point_estimate,
			observations: `${sample.iters.length} Criterion samples`,
			upper: (elementsPerIteration * 1e9) / estimate.confidence_interval.lower_bound,
		}
		report(benchmarkId, "native", unit, throughput)
	} finally {
		rmSync(outputDirectory, { force: true, recursive: true })
	}
}

function runWasmBenchmark(benchmarkName: BenchmarkName, unit: "moves" | "nodes") {
	run("pnpm", ["run", "build:game-server:benchmark"])

	const outputDirectory = mkdtempSync(join(tmpdir(), "chess-benchmark-"))
	const outputPath = join(outputDirectory, "result.json")

	try {
		run("pnpm", [
			"vitest",
			"bench",
			"--run",
			`game-server/benches/${benchmarkName}.bench.ts`,
			"--config",
			"vitest.bench.config.ts",
			"--outputJson",
			outputPath,
		])

		const benchmarkOutput = vitestOutputSchema.parse(readJson(outputPath))
		const benchmarkResults = benchmarkOutput.files.flatMap((file) =>
			file.groups.flatMap((group) => group.benchmarks),
		)
		const [benchmarkResult] = z.tuple([vitestBenchmarkSchema]).parse(benchmarkResults)

		const match = new RegExp(`^(.+)/${unit}-(\\d+)$`).exec(benchmarkResult.name)
		const benchmarkId = match?.[1]
		const elementsPerIteration = Number(match?.[2])
		if (!benchmarkId || !Number.isFinite(elementsPerIteration)) {
			throw new Error(`Invalid WASM benchmark name: ${benchmarkResult.name}`)
		}

		const point = benchmarkResult.hz * elementsPerIteration
		const margin = (point * benchmarkResult.rme) / 100
		const throughput = {
			lower: point - margin,
			point,
			observations: `${benchmarkResult.sampleCount} iterations`,
			upper: point + margin,
		}
		report(benchmarkId, "wasm", unit, throughput)
	} finally {
		rmSync(outputDirectory, { force: true, recursive: true })
	}
}

function run(command: string, args: string[], env = process.env) {
	const result = spawnSync(command, args, { encoding: "utf8", env })

	if (result.error) {
		throw result.error
	}

	if (result.status !== 0) {
		process.stdout.write(result.stdout)
		process.stderr.write(result.stderr)
		throw new Error(`${command} exited with status ${result.status ?? "unknown"}`)
	}
}

function findCriterionResult(outputDirectory: string, benchmarkId: string) {
	for (const entry of readdirSync(outputDirectory, { encoding: "utf8", recursive: true })) {
		if (!entry.endsWith("new/benchmark.json")) continue

		const path = join(outputDirectory, entry)
		const benchmarkResult = criterionBenchmarkSchema.safeParse(readJson(path))
		if (benchmarkResult.success && benchmarkResult.data.full_id === benchmarkId) {
			return join(path, "..")
		}
	}

	throw new Error(`Could not find Criterion results for ${benchmarkId}`)
}

function readJson(path: string) {
	return JSON.parse(readFileSync(path, "utf8"))
}

function report(
	benchmarkId: string,
	target: BenchmarkTarget,
	unit: "moves" | "nodes",
	throughput: { lower: number; observations: string; point: number; upper: number },
) {
	const margin = ((throughput.upper - throughput.lower) / 2 / throughput.point) * 100
	console.log(`\n${benchmarkId} (${target})`)
	console.log(
		`${throughputFormatter.format(throughput.point)} ${unit}/s ±${margin.toFixed(2)}% (${throughput.observations})`,
	)
}
