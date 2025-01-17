import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		maxWorkers: 10,
		minWorkers: 1,

		hideSkippedTests: true,
		retry: 1,
		setupFiles: './setup.ts',
	},
})
