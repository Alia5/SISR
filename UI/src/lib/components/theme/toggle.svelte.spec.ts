import { tick } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import '../../../css/main.pcss';

let mockTheme: 'dark' | 'light' | undefined;

vi.mock('$app/state', () => ({
    page: {
        get data() {
            return { theme: mockTheme };
        }
    }
}));

import ThemeToggle from './toggle.svelte';

describe('ThemeToggle', () => {
    interface TestCase {
        name: string;
        setupPageData?: () => { theme: 'dark' | 'light' | undefined };
        prefersColorScheme: 'dark' | 'light';
        action?: () => Promise<void>;
        expected: {
            darkMode: boolean;
            colorScheme: '' | 'light' | 'dark';
        };
    }

    const testCases: TestCase[] = [
        {
            name: 'SUCCESS_UseSavedDarkTheme_OverridesBrowserPreference',
            setupPageData: () => ({ theme: 'dark' }),
            prefersColorScheme: 'light',
            expected: {
                darkMode: true,
                colorScheme: ''
            }
        }
        // {
        //     name: 'SUCCESS_UseSavedLightTheme_OverridesBrowserPreference',
        //     setupPageData: () => ({ theme: 'light' }),
        //     prefersColorScheme: 'dark',
        //     expected: {
        //         darkMode: false,
        //         colorScheme: ''
        //     }
        // },
        // {
        //     name: 'SUCCESS_UseBrowserDarkPreference_WhenNoSavedTheme',
        //     prefersColorScheme: 'dark',
        //     expected: {
        //         darkMode: true,
        //         colorScheme: ''
        //     }
        // },
        // {
        //     name: 'SUCCESS_UseBrowserLightPreference_WhenNoSavedTheme',
        //     prefersColorScheme: 'light',
        //     expected: {
        //         darkMode: false,
        //         colorScheme: ''
        //     }
        // },
        // {
        //     name: 'SUCCESS_ToggleDarkToLight',
        //     setupPageData: () => ({ theme: 'dark' }),
        //     prefersColorScheme: 'dark',
        //     action: async () => {
        //         const checkboxEl = page.getByRole('checkbox', { name: 'Theme Toggle' });
        //         await checkboxEl.click();
        //         await new Promise((resolve) => setTimeout(resolve, 1000));
        //     },
        //     expected: {
        //         darkMode: false,
        //         colorScheme: 'light'
        //     }
        // },
        // {
        //     name: 'SUCCESS_ToggleLightToDark',
        //     setupPageData: () => ({ theme: 'light' }),
        //     prefersColorScheme: 'light',
        //     action: async () => {
        //         const checkboxEl = page.getByRole('checkbox', { name: 'Theme Toggle' });
        //         await checkboxEl.click();
        //         await new Promise((resolve) => setTimeout(resolve, 1000));
        //     },
        //     expected: {
        //         darkMode: true,
        //         colorScheme: 'dark'
        //     }
        // }
    ];

    testCases.forEach((tc) => {
        it(tc.name, async () => {
            document.documentElement.style.colorScheme = '';

            const matchMediaMock = vi.fn((query: string) => ({
                matches: query.includes('dark') ? tc.prefersColorScheme === 'dark' : false,
                media: query,
                onchange: null,
                addListener: vi.fn(),
                removeListener: vi.fn(),
                addEventListener: vi.fn(),
                removeEventListener: vi.fn(),
                dispatchEvent: vi.fn()
            }));
            vi.stubGlobal('matchMedia', matchMediaMock);

            const doc = document as Document & { startViewTransition?: Document['startViewTransition'] };
            Reflect.deleteProperty(doc, 'startViewTransition');

            let pageData: { theme: 'dark' | 'light' | undefined } | undefined;
            if (tc.setupPageData) {
                pageData = tc.setupPageData();
                mockTheme = pageData.theme;
            } else {
                mockTheme = undefined;
            }

            const { unmount } = render(ThemeToggle);

            await tick();

            const expectedInitialDarkMode = pageData?.theme
                ? pageData.theme === 'dark'
                : tc.prefersColorScheme === 'dark';

            {
                const toggleInput = page.getByRole('checkbox', { name: 'Theme Toggle' });
                const checkbox = toggleInput.element() as HTMLInputElement;
                expect(checkbox.checked).toBe(expectedInitialDarkMode);
            }

            if (tc.action) {
                await tc.action();
            }

            await tick();


            {
                const toggleInput = page.getByRole('checkbox', { name: 'Theme Toggle' });
                const checkbox = toggleInput.element() as HTMLInputElement;
                expect(checkbox.checked).toBe(tc.expected.darkMode);

                const colorScheme = document.documentElement.style.colorScheme;
                expect(colorScheme).toBe(tc.expected.colorScheme);
            }


            await unmount();
            vi.unstubAllGlobals();
            vi.restoreAllMocks();
        });
    });
});
