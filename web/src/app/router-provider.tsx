"use client"

import { useRouter } from "next/navigation"
import { ReactNode } from "react"
import { RouterProvider as RacRouterProvider } from "react-aria-components"

declare module "react-aria-components" {
	interface RouterConfig {
		href: Parameters<ReturnType<typeof useRouter>["push"]>[0]
		routerOptions: NonNullable<Parameters<ReturnType<typeof useRouter>["push"]>[1]>
	}
}

export function RouterProvider({ children }: { children: ReactNode }) {
	const router = useRouter()
	return <RacRouterProvider navigate={router.push}>{children}</RacRouterProvider>
}
