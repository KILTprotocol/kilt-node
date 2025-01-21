import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		maxWorkers: process.env.CI ? 4 : 10,
		minWorkers: process.env.CI ? 1 : 5,
		hideSkippedTests: true,
		retry: process.env.CI ? 3 : 0,
		setupFiles: './setup.ts',
	},
})
