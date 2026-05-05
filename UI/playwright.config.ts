import { defineConfig } from '@playwright/test';

import { existsSync } from 'fs';

const chromiumPath = existsSync('/usr/bin/chromium') ? '/usr/bin/chromium' : undefined;

export default defineConfig({
    webServer: { command: 'npm run build && npm run preview', port: 4173 },
    testDir: 'e2e',
    use: {
        launchOptions: chromiumPath ? { executablePath: chromiumPath } : {}
    }
});
