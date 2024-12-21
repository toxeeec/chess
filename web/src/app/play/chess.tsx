"use client"

import { Piece } from "./piece"
import { Square } from "./square"
import { SQUARES } from "./square"
import { useChess } from "./use-chess"
import { restrictToParentElement } from "@/utils"
import { DndContext, DragEndEvent, useDraggable, useDroppable } from "@dnd-kit/core"
import { snapCenterToCursor } from "@dnd-kit/modifiers"
import { CSS } from "@dnd-kit/utilities"
import { User } from "lucide-react"
import { useId } from "react"
import { twJoin } from "tailwind-merge"

export function Chess({ white, black }: { white: string; black: string }) {
	const { pieces, movePiece, boardRef } = useChess()
	const id = useId()

	const handleDragEnd = (e: DragEndEvent) => {
		if (!e.over) return
		movePiece(e.active.id as Square, e.over.id as Square)
	}

	return (
		<div className="flex size-board-container flex-col items-center gap-3">
			<PlayerInfo name={black} />
			<DndContext
				id={id}
				modifiers={[snapCenterToCursor, restrictToParentElement]}
				onDragEnd={handleDragEnd}
			>
				<div ref={boardRef} className="grid size-board grid-cols-8 grid-rows-8 rounded-lg">
					{pieces.map(([square, piece]) => (
						<DraggablePiece key={square} square={square} piece={piece} />
					))}
					{SQUARES.map((square) => (
						<DroppableSquare key={square} square={square} />
					))}
				</div>
			</DndContext>
			<PlayerInfo name={white} />
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
			className={twJoin(isDragging ? "z-20" : "z-10", isDragging && "cursor-grabbing")}
			ref={setNodeRef}
			style={{
				backgroundImage: `url(${Piece.imagePath(piece)})`,
				gridArea: Square.gridArea(square),
				transform: CSS.Translate.toString(transform),
			}}
		/>
	)
}

function DroppableSquare({ square }: { square: Square }) {
	const { isOver, setNodeRef } = useDroppable({ id: square })

	const isLight = Square.isLight(square)

	return (
		<div
			className={twJoin(
				isLight ? "bg-neutral-400" : "bg-neutral-700",
				isOver && "border-square border-neutral-500",
			)}
			ref={setNodeRef}
			style={{ gridArea: Square.gridArea(square), ...Square.borderRadiusStyles(square) }}
		></div>
	)
}

function PlayerInfo({ name }: { name: string }) {
	return (
		<div className="flex w-board items-start gap-2 overflow-hidden font-semibold text-neutral-200">
			<User className="size-10 flex-shrink-0 rounded-md bg-neutral-700 stroke-neutral-400 p-1" />
			<span className="truncate">{name}</span>
		</div>
	)
}
