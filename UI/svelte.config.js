import adapter from '@sveltejs/adapter-static';
//import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from "mdsvex";
import rehypeExternalLinks from 'rehype-external-links';
import remarkGfm from "remark-gfm";


/** @type {import('@sveltejs/kit').Config} */
const config = {
    compilerOptions: {
        runes: true,
        experimental: {
            async: true
        },
    },
    kit: {
        adapter: adapter({
            fallback: 'index.html'
        })
    },
    extensions: [".svelte", ".svx"],
    preprocess: [
        vitePreprocess({
            style: true,
        }),
        mdsvex({
            remarkPlugins: [
                [remarkGfm, {

                }]
            ],
            rehypePlugins: [
                [rehypeExternalLinks, { rel: ['nofollow'], target: '_blank' }]
            ]
        })
    ]
};

export default config;
