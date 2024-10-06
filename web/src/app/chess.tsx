"use client"

import { Piece } from "./piece"
import { Square, SQUARES } from "./square"
import { useChess } from "./use-chess"
import { restrictToParentElement } from "@/utils"
import { DndContext, DragEndEvent, useDraggable, useDroppable } from "@dnd-kit/core"
import { snapCenterToCursor } from "@dnd-kit/modifiers"
import { CSS } from "@dnd-kit/utilities"
import { useRouter } from "next/navigation"
import { useEffect, useId } from "react"
import { io } from "socket.io-client"
import { twJoin } from "tailwind-merge"

export function Chess({ gameId }: { gameId: string }) {
	const { pieces, movePiece, boardRef } = useChess()
	const id = useId()
	const router = useRouter()

	const handleDragEnd = (e: DragEndEvent) => {
		if (!e.over) return
		movePiece(e.active.id as Square, e.over.id as Square)
	}

	useEffect(() => {
		const socket = io(process.env.NEXT_PUBLIC_GAME_SERVER_URL)

		socket.on("connect", () => {
			socket.emit("join", gameId)
		})

		socket.on("message", (e) => {
			console.log(e)
		})

		socket.on("disconnect", () => {
			router.replace("/play")
		})
	}, [gameId, router])

	return (
		<DndContext
			id={id}
			modifiers={[snapCenterToCursor, restrictToParentElement]}
			onDragEnd={handleDragEnd}
		>
			<div ref={boardRef} className="grid size-board grid-cols-8 grid-rows-8">
				{pieces.map(([square, piece]) => (
					<DraggablePiece key={square} square={square} piece={piece} />
				))}
				{SQUARES.map((square) => (
					<DroppableSquare key={square} square={square} />
				))}
			</div>
		</DndContext>
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
			className={twJoin("z-10", isDragging && "z-20 cursor-grabbing")}
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
				"border-square border-neutral-500",
				isLight ? "bg-neutral-400" : "bg-neutral-700",
				!isOver && "border-none",
			)}
			ref={setNodeRef}
			style={{ gridArea: Square.gridArea(square) }}
		></div>
	)
}
