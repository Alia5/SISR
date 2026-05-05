import { log } from '$lib/log';
import createClient, { type FetchResponse } from 'openapi-fetch';
import type { paths } from './openapi';
import { error } from '@sveltejs/kit';

const apiURL = `${window.location.protocol}//localhost:${
    Number.parseInt(document?.documentElement?.getAttribute('data-api-port') ?? '', 10) || 5746
}`;

log.debug('Creating API client with', 'url', apiURL);
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


export interface APIError {
    detail?: string;
    errors?: {
        location?: string;
        message?: string;
        value?: unknown;
    }[] | null;
    instance?: string;
    status?: number;
    title?: string;
    type: string;
}

export const isApiError = (e: unknown): e is APIError => typeof e === 'object' && e !== null && 'type' in e && typeof e.type === 'string';

export const hasStatus = (e: unknown): e is {
    status: number;
} => typeof e === 'object' && e !== null && 'status' in e && typeof (e as Record<string, unknown>).status === 'number';


export const FALLBACK_MESSAGE = 'An unknown error occurred';

export const wrapClientError = async <R extends { data?: unknown; error?: unknown }>(
    promise: Promise<R> | (() => Promise<R>),
    fallbackStatus = 500,
    fallbackMessage = FALLBACK_MESSAGE
): Promise<NonNullable<R['data']>> => {
    try {
        const { data, error: err } = await (typeof promise === 'function' ? promise() : promise);
        if (err) {
            throw err;
        }
        return data as NonNullable<R['data']>;
    } catch (e) {
        log.error('API request failed', 'error', e);
        error(
            hasStatus(e) ? e.status : fallbackStatus,
            isApiError(e)
                ? { ...e, message: e.title ?? fallbackMessage }
                : { message: fallbackMessage }
        );
    }
};
