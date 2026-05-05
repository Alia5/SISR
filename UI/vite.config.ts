import svg from '@poppanator/sveltekit-svg';
import { enhancedImages } from '@sveltejs/enhanced-img';
import { sveltekit } from '@sveltejs/kit/vite';
import { playwright } from '@vitest/browser-playwright';
import { existsSync } from 'fs';
import Unfonts from 'unplugin-fonts/vite';
import Icons from 'unplugin-icons/vite';
import { defineConfig } from 'vitest/config';


const chromiumPath = (() =>existsSync('/usr/bin/chromium') ? '/usr/bin/chromium' : undefined
)();

let svgoPrefixIdsCount = 0;
export default defineConfig({
    server: {
        allowedHosts: ['host.docker.internal','steaminputdb.local','dev.local', '*']
    },
    plugins: [
        enhancedImages(),
        sveltekit(),
        svg({
            includePaths: [
                './src/static/',
                './src/lib/assets/',
                './src/lib/components/layout_preview/controllers/',
                './src/lib/components/layout_preview/glyphs/'
            ],
            svgoOptions: {
                plugins: [
                    'preset-default',
                    {
                        name: 'prefixIds',
                        params: {
                            delim: '',
                            prefix: () => svgoPrefixIdsCount++
                        }
                    // eslint-disable-next-line @typescript-eslint/no-explicit-any
                    } as any
                ]
            }
        }),
        Icons({
            compiler: 'svelte',
            autoInstall: true
        }),
        Unfonts({
            google: {
                display: 'block',
                families: [
                    {
                        defer: false,
                        name: 'Noto Sans',
                        styles: 'ital,wdth,wght@0,62.5..100,100..900;1,62.5..100,100..900'
                    },
                    {
                        name: 'Noto Color Emoji',
                        defer: false,
                        styles: 'wght@400'
                    },
                    {
                        name: 'Roboto',
                        defer: false,

                        styles: 'ital,wght@0,400;1,200'
                    }
                ]
            }
        })
    ],
    test: {
        expect: { requireAssertions: true },
        projects: [
            {
                extends: './vite.config.ts',
                test: {
                    name: 'client',
                    browser: {
                        enabled: true,
                        provider: playwright({
                            launchOptions: chromiumPath ? {
                                executablePath: chromiumPath

                            } : {}
                        }),
                        instances: [{ browser: 'chromium', headless: process.env.TEST_SHOW_BROWSER ? false : true }]
                    },
                    include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
                    exclude: ['src/lib/server/**']
                }
            },

            {
                extends: './vite.config.ts',
                test: {
                    name: 'server',
                    environment: 'node',
                    include: ['src/**/*.{test,spec}.{js,ts}'],
                    exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
                }
            }
        ]
    }
});

