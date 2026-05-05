import { clientWithSvelteFetch, wrapClientError } from '$lib/api/client';
import { log } from '$lib/log';

export const load = async ({ fetch }) => {

    const client = clientWithSvelteFetch(fetch);

    log.debug('Fetching steam status...');
    const [steamStatus, inputInfo] = await Promise.all([
        wrapClientError(client.GET('/api/v1/steam_status', {
            signal: AbortSignal.timeout(5000)
        })),
        wrapClientError(client.GET('/api/v1/input_info', {
            signal: AbortSignal.timeout(5000)
        }))
    ]);

    return {
        steamStatus: steamStatus,
        inputInfo: inputInfo
    };
};
