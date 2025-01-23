import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		maxWorkers: process.env.CI ? 4 : 10,
		minWorkers: process.env.CI ? 1 : 5,
		hideSkippedTests: true,
		retry: process.env.CI ? 3 : 0,
		setupFiles: './src/setup.ts',
		hookTimeout: process.env.CI ? 120_000 : 30_000,
		testTimeout: process.env.CI ? 60_000 : 10_000,
		teardownTimeout: process.env.CI ? 60_000 : 10_000,
	},
})
