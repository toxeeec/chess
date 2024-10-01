import { Chess } from "./chess"

export default function Home() {
	return (
		<div className="grid h-full place-items-center">
			<div className="grid size-4/5 place-items-center">
				<Chess />
			</div>
		</div>
	)
}
