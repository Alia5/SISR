import stylistic from '@stylistic/eslint-plugin';
import { defineConfig } from "eslint/config";
import { fileURLToPath } from "url";


const __filename = fileURLToPath(import.meta.url);

import js from "@eslint/js";
import eslintPluginPrettier from "eslint-plugin-prettier/recommended";
import eslintPluginSvelte from 'eslint-plugin-svelte';
import globals from "globals";
import svelteEslintParser from 'svelte-eslint-parser';
import tseslint from 'typescript-eslint';


export default defineConfig(
    eslintPluginPrettier,
    js.configs.recommended,
    ...tseslint.configs.recommended,
    ...eslintPluginSvelte.configs['flat/recommended'],
    {
        ignores: [
            '*.cjs',
            '*.html',
            'postcss.config.js',
            'svelte.config.js',
            'eslint.config.js',
            ".svelte-kit/**/*",
            "build/**/*",
            "**/*/openapi.d.ts"
        ],
    },
    {
        files: ['**/*.ts', '**/*.tsx', '**/*.js', '**/*.jsx'],
        ignores: ['e2e/**/*', 'playwright.config.ts'],
        plugins: {
            '@typescript-eslint': tseslint.plugin,
            '@stylistic': stylistic
        },

        languageOptions: {
            parser: tseslint.parser,
            globals: {
                ...globals.browser,
                ...globals.node,
                ...globals.es2021,

            },
            parserOptions: {
                sourceType: 'module',
                ecmaVersion: 2020,
                project: './tsconfig.json',
            },
        },
        rules: {
            // typescript-eslint strongly recommend that you do not use the no-undef lint rule on TypeScript projects.
            // see: https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
            'no-undef': 'off',
            "prettier/prettier": "off",
            '@typescript-eslint/no-namespace': 'off',
            // '@typescript-eslint/ban-types': 'error',
            '@typescript-eslint/adjacent-overload-signatures': 'error',
            '@typescript-eslint/array-type': 'error',
            '@typescript-eslint/consistent-type-definitions': ['error', 'interface'],
            '@typescript-eslint/no-inferrable-types': 'error',
            '@typescript-eslint/no-misused-new': 'error',
            '@typescript-eslint/no-this-alias': 'error',
            '@typescript-eslint/prefer-for-of': 'error',
            '@typescript-eslint/prefer-function-type': 'error',
            '@typescript-eslint/prefer-namespace-keyword': 'error',
            'no-inner-declarations': 'off', // we have es6blocked scoped functions.
            '@typescript-eslint/triple-slash-reference': 'error',
            '@stylistic/type-annotation-spacing': 'error',
            '@typescript-eslint/unified-signatures': 'error',
            '@typescript-eslint/no-explicit-any': 'error',
            '@typescript-eslint/no-unused-vars': 'error',
            '@typescript-eslint/unbound-method': 'warn',
            '@typescript-eslint/no-floating-promises': 'error',
            '@typescript-eslint/no-unnecessary-type-assertion': 'error',
            'object-curly-spacing': ['error', 'always'],
            '@stylistic/semi': [
                'error',
                'always'
            ],
            '@stylistic/quotes': [
                'warn',
                'single'
            ],
            '@stylistic/member-delimiter-style': [
                'error',
                {
                    multiline: {
                        delimiter: 'semi',
                        requireLast: true
                    },
                    singleline: {
                        delimiter: 'semi',
                        requireLast: false
                    }
                }
            ],
            '@stylistic/indent': [
                'warn',
                4,
                {
                    FunctionDeclaration: {
                        parameters: 'first'
                    },
                    FunctionExpression: {
                        parameters: 'first'
                    },
                    SwitchCase: 1
                }
            ],

            '@typescript-eslint/explicit-member-accessibility': [
                'error',
                {
                    accessibility: 'explicit'
                }
            ],
            '@typescript-eslint/no-use-before-define': ['error', { functions: false }],
            'no-console': 'off',
            'no-return-await': 'error',
            'arrow-body-style': 'error',
            'arrow-parens': [
                'error',
                'always'
            ],
            'comma-dangle': [
                'error',
                {
                    objects: 'never',
                    arrays: 'never',
                    functions: 'never'
                }
            ],
            // 'prefer-arrow/prefer-arrow-functions': 'error',
            'prefer-arrow-callback': 'error',
            'prefer-const': 'error',
            'quote-props': [
                'error',
                'consistent-as-needed'
            ],
            'no-var': 'error',
            'new-parens': 'error',
            'no-caller': 'error',
            'no-cond-assign': 'error',
            'no-debugger': 'error',
            'no-empty': 'error',
            'no-eval': 'error',
            'no-multiple-empty-lines': 'warn',
            'no-new-wrappers': 'error',
            'no-redeclare': 'error',
            'no-shadow': [
                'error',
                {
                    hoist: 'all'
                }
            ],
            'no-throw-literal': 'error',
            'no-trailing-spaces': 'error',
            'no-undef-init': 'error',
            'no-underscore-dangle': 'off',
            'no-unsafe-finally': 'error',
            'no-unused-labels': 'error',
            'spaced-comment': 'error',
            'use-isnan': 'error',
            'max-len': [
                'warn',
                {
                    code: 140
                }
            ],
            'dot-notation': 'error',
            'eqeqeq': 'error',
            'eol-last': 'error',
            'linebreak-style': ['error', 'unix'],
            'block-spacing': ['error', 'always'],
            // 'import/no-deprecated': 'warn', // eslint deprecation rule sucks. just wrns on deprecated IMPORTs
            'tsdoc/syntax': 'off'

            // Place to specify ESLint rules. Can be used to overwrite rules specified from the extended configs
            // e.g. '@typescript-eslint/explicit-function-return-type': 'off',
        },
    },
    {
        files: ['*.svelte', '**/*.svelte'],
        languageOptions: {
            parser: svelteEslintParser,
            parserOptions: {
                parser: tseslint.parser,
            },
            globals: {
                ...globals.browser,
                ...globals.node,
                ...globals.es2021,

            },
        },
        rules: {
            'prettier/prettier': ['warn', {
                "svelteStrictMode": true,
                "svelteBracketNewLine": false,
                "svelteAllowShorthand": true,
                "svelteIndentScriptAndStyle": false,
                "tabWidth": 4,
                "bracketSpacing": true,
                "trailingComma": "none",
                "arrowParens": "always",
                "semi": true,
                "singleQuote": true,
                "printWidth": 110,
                "proseWrap": "preserve",
                "plugins": ["prettier-plugin-svelte"],
            }],
            '@typescript-eslint/no-unused-vars': ['error', {
                varsIgnorePattern: '^_',
                argsIgnorePattern: '^_',
                caughtErrorsIgnorePattern: '^_',

            }],
        }
    },
    {
        files: ['e2e/**/*.ts', 'playwright.config.ts'],
        plugins: {
            '@typescript-eslint': tseslint.plugin,
            '@stylistic': stylistic
        },
        languageOptions: {
            parser: tseslint.parser,
            globals: {
                ...globals.browser,
                ...globals.node,
                ...globals.es2021,
            },
            parserOptions: {
                sourceType: 'module',
                ecmaVersion: 2020,
            },
        },
    }
);