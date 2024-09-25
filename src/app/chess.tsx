"use client"

import { Piece } from "./piece"
import { Square, SQUARES } from "./square"
import { useChessState } from "./use-chess-state"
import { twMerge } from "@/utils"
import { DndContext, DragEndEvent, useDraggable, useDroppable } from "@dnd-kit/core"
import { restrictToParentElement, snapCenterToCursor } from "@dnd-kit/modifiers"
import { CSS } from "@dnd-kit/utilities"
import { ReactNode, useId } from "react"

export function Chess() {
	const { pieces, movePiece } = useChessState()
	const id = useId()

	const handleDragEnd = (e: DragEndEvent) => {
		if (!e.over) return
		movePiece(e.active.id as Square, e.over.id as Square)
	}

	return (
		<DndContext
			id={id}
			modifiers={[snapCenterToCursor, restrictToParentElement]}
			onDragEnd={handleDragEnd}
		>
			<Board>
				{pieces.map(([square, piece]) => (
					<DraggablePiece key={square} square={square as Square} piece={piece} />
				))}
				{SQUARES.map((square) => (
					<DroppableSquare key={square} square={square} />
				))}
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

function DraggablePiece({ square, piece }: { square: Square; piece: Piece }) {
	const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
		id: square,
	})

	return (
		<button
			{...listeners}
			{...attributes}
			className={twMerge("z-10", isDragging && "z-20 cursor-grabbing")}
			ref={setNodeRef}
			style={{
				backgroundImage: `url(${Piece.imagePath(piece)})`,
				gridColumn: Square.file(square) + 1,
				gridRow: Square.rank(square) + 1,
				transform: CSS.Translate.toString(transform),
			}}
		/>
	)
}

function DroppableSquare({ square }: { square: Square }) {
	const { setNodeRef } = useDroppable({ id: square })
	return (
		<div
			ref={setNodeRef}
			style={{
				gridColumn: Square.file(square) + 1,
				gridRow: Square.rank(square) + 1,
			}}
		></div>
	)
}
