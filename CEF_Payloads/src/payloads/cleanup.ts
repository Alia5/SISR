Object.entries(window.__sisrCleanup || {}).forEach(([key, cleanupFn]) => {
    try {
        cleanupFn();
    } catch (e) {
        console.error('Error during SISR cleanup', e);
    }
});
window.__sisrCleanup = {};
