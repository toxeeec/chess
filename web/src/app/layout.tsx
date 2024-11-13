import "./globals.css"
import { RouterProvider } from "./router-provider"
import { ReactNode } from "react"

export const metadata = {
	title: "chess",
}

export default function RootLayout({ children }: { children: ReactNode }) {
	return (
		<RouterProvider>
			<html lang="en">
				<body className="h-dvh bg-neutral-900">{children}</body>
			</html>
		</RouterProvider>
	)
}
