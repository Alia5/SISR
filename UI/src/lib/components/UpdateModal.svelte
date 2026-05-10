<script lang="ts">
import { client, wrapClientError } from '$lib/api/client';
import type { components } from '$lib/api/openapi';
import Modal from '$lib/components/Modal.svelte';
import { toast } from '$lib/toaster/toaster.svelte';
import { fade } from 'svelte/transition';

let {
	show = $bindable(false),
	updateInfo
}: {
	show?: boolean;
	updateInfo: components['schemas']['UpdateStateResponse'];
} = $props();
</script>

<Modal bind:open={show}>
	<div class="card glass dialog-content" transition:fade>
		<h2>Update Available</h2>
		<div>
			<p>A new version of SISR ({updateInfo.update_version}) is available.</p>
			<div class="button-group">
				<button
					name="skip"
					class="negative"
					onclick={() => {
						wrapClientError(client.POST('/api/v1/update/skip'))
							.catch((e) => {
								toast({
									color: 'firebrick',
									message: `Failed to skip this version.\n Error: ${e}`
								});
							})
							.finally(() => {
								show = false;
							});
					}}>Skip this version</button>
				<button
					name="remind-me"
					class="negative"
					onclick={() => {
						wrapClientError(client.POST('/api/v1/update/remind-later'))
							.catch((e) => {
								toast({
									color: 'firebrick',
									message: `Failed to set remind later.\n Error: ${e}`
								});
							})
							.finally(() => {
								show = false;
							});
					}}>Remind me later</button>
				<button
					name="view-on-github"
					style="background-color: rgb(128 128 200);"
					onclick={() => {
						wrapClientError(client.POST('/api/v1/update/view-on-github')).catch((e) => {
							toast({
								color: 'firebrick',
								message: `Failed to open GitHub page.\n Error: ${e}`
							});
						});
					}}>View on GitHub</button>
				<button
					name="update-now"
					onclick={() => {
						wrapClientError(client.POST('/api/v1/update/install'))
							.catch((e) => {
								toast({
									color: 'firebrick',
									message: `Failed to install update.\n Error: ${e}`
								});
							})
							.finally(() => {
								show = false;
							});
					}}>Update Now</button>
			</div>
		</div>
	</div>
</Modal>

<style lang="postcss">
.dialog-content {
	display: grid;
	place-self: center;
	position: absolute;
	top: 50%;
	left: 50%;
	transform: translate(-50%, -50%);
	gap: 1em;
}

.button-group {
	display: flex;
	flex-flow: row wrap;
	gap: 1em;
	justify-content: end;
	margin-top: 1em;
}

button {
	padding: 1em 2em;
	white-space: nowrap;
}
button.negative {
	background-color: rgb(128 128 128);
}

em {
	color: var(--color-muted);
}
</style>
