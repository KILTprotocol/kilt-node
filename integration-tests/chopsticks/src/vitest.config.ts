import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		maxWorkers: process.env.CI ? 4 : 10,
		minWorkers: process.env.CI ? 1 : 5,
		hideSkippedTests: true,
		retry: process.env.CI ? 3 : 0,
		setupFiles: './src/setup.ts',
		hookTimeout: 120_000,
		testTimeout: 60_000,
		teardownTimeout: 60_000,
	},
})
