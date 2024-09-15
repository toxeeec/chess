import { Board } from "./board"
import { FILES, Piece } from "./piece"

export default function Home() {
	return (
		<div className="grid h-full place-items-center">
			<Board>
				<Piece type="ROOK" color="BLACK" square="A8" />
				<Piece type="KNIGHT" color="BLACK" square="B8" />
				<Piece type="BISHOP" color="BLACK" square="C8" />
				<Piece type="QUEEN" color="BLACK" square="D8" />
				<Piece type="KING" color="BLACK" square="E8" />
				<Piece type="BISHOP" color="BLACK" square="F8" />
				<Piece type="KNIGHT" color="BLACK" square="G8" />
				<Piece type="ROOK" color="BLACK" square="H8" />

				{FILES.map((file) => (
					<Piece key={file} type="PAWN" color="BLACK" square={`${file}7`} />
				))}
				{FILES.map((file) => (
					<Piece key={file} type="PAWN" color="WHITE" square={`${file}2`} />
				))}

				<Piece type="ROOK" color="WHITE" square="A1" />
				<Piece type="KNIGHT" color="WHITE" square="B1" />
				<Piece type="BISHOP" color="WHITE" square="C1" />
				<Piece type="QUEEN" color="WHITE" square="D1" />
				<Piece type="KING" color="WHITE" square="E1" />
				<Piece type="BISHOP" color="WHITE" square="F1" />
				<Piece type="KNIGHT" color="WHITE" square="G1" />
				<Piece type="ROOK" color="WHITE" square="H1" />
			</Board>
		</div>
	)
}
