import { defineConfig } from 'vite'

export default defineConfig({
	test: {
		maxWorkers: 10,
		minWorkers: 1,
		dangerouslyIgnoreUnhandledErrors: true,
		hideSkippedTests: true,
		retry: 3,
	},
})
