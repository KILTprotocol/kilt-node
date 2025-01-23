// peer dependencies can throw unhandled promise rejections. This is a workaround to ignore them.
process.on('unhandledRejection', (reason, promise) => {
	// Ignore unhandled promise rejections most likely emitted from peer dependencies
	if (!process.env.CI) {
		console.warn('Unhandled Rejection:', reason, 'Promise:', promise)
	}
})
