import type { NextConfig } from "next"

const nextConfig: NextConfig = {
	experimental: {
		reactCompiler: true,
		typedRoutes: true,
		typedEnv: true,
	},
}

export default nextConfig
