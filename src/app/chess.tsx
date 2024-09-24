"use client"

import { Piece, FILES } from "./piece"
import { DndContext } from "@dnd-kit/core"
import { restrictToParentElement, snapCenterToCursor } from "@dnd-kit/modifiers"
import { ReactNode, useId } from "react"

export function Chess() {
	const id = useId()

	return (
		<DndContext id={id} modifiers={[snapCenterToCursor, restrictToParentElement]}>
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
		</DndContext>
	)
}

function Board({ children }: { children: ReactNode }) {
	return (
		<div className="relative grid aspect-square w-board grid-cols-8 grid-rows-8">
			{children}
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 8 8"
				className="absolute inset-0 -z-10"
			>
				<path className="fill-neutral-700" d="M0 0h8v8H0" />
				<path
					className="fill-neutral-400"
					d="M0 0h8v1H0m0 1h8v1H0m0 1h8v1H0m0 1h8v1H0m1-7v8h1V0m1 0v8h1V0m1 0v8h1V0m1 0v8h1V0"
				/>
			</svg>
		</div>
	)
}
