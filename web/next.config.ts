import type { NextConfig } from "next"

const nextConfig: NextConfig = {
	async redirects() {
		return [
			{
				source: "/",
				destination: "/play",
				permanent: false,
			},
		]
	},
	experimental: {
		reactCompiler: true,
		typedRoutes: true,
		typedEnv: true,
	},
}

export default nextConfig
