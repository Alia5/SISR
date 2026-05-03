import createClient, { type FetchResponse } from 'openapi-fetch';
import type { paths } from './openapi';

const apiURL = window.__SISR_API_URL;

export const client = createClient<paths>({
    baseUrl: apiURL
});


export const clientWithSvelteFetch = (fetch: typeof window.fetch, url?: string) => createClient<paths>({
    baseUrl: url || apiURL,
    fetch
});

export type ResponseType<
    M extends keyof typeof client,
    P extends keyof paths,
> = P extends keyof paths
    ? Lowercase<M> extends keyof paths[P]
        ? paths[P][Lowercase<M>] extends Record<string | number, unknown>
            ? FetchResponse<paths[P][Lowercase<M>], unknown, `${string}/${string}`>
            : never
        : never
    : never;

