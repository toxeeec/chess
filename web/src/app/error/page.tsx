export default async function Error({
	searchParams,
}: {
	searchParams: Promise<{ status?: string; message?: string }>
}) {
	const { status, message = "unknown error" } = await searchParams
	return (
		<main className="flex h-full items-center justify-center">
			<h1 className="absolute text-[5vmin] text-neutral-200">{message}</h1>
			<span className="absolute -z-10 select-none text-center text-[40vmin] font-semibold text-neutral-800 opacity-30">
				{status}
			</span>
		</main>
	)
}
