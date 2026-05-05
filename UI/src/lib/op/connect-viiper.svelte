<script lang="ts">
import { client, wrapClientError } from '$lib/api/client';
import { log } from '$lib/log';

export const connect = async () => {
	wrapClientError(client.POST('/api/v1/connect_viiper'))
		.then(() => {
			log.info('Connected to VIIPER successfully');
		})
		.catch((e) => {
			// TODO: match errors and show user dialog.
			log.error('Failed to connect to VIIPER', 'error', e);
		});
};
</script>
