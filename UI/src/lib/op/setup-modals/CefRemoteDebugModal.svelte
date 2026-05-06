<script lang="ts">
import Modal from '$lib/components/Modal.svelte';
import { client, wrapClientError } from '$lib/api/client';
import { toast } from '$lib/toaster/toaster.svelte';
import { invalidateAll } from '$app/navigation';
import Spinner from '$lib/components/Spinner.svelte';
import { fade } from 'svelte/transition';

let {
	modal = $bindable(),
	show = $bindable(false)
}: {
	modal?: Modal;
	show?: boolean;
} = $props();

let loading = $state(false);
</script>

<Modal bind:this={modal} open={show}>
	<div class="card glass dialog-content" transition:fade>
		<h2>Enable Steam CEF debugging</h2>
		<div>
			<p>SISR <strong>requires</strong> advanced access to Steam for full functionality.</p>
			<p>This can be done via Steam CEF debugging, a developer utility exposed by Steam.</p>
			<p>Should we try to enable this feature now?</p>
			<p><em>A UAC prompt may appear and Steam will be restarted after this.</em></p>
			<div class="button-group">
				<button
					name="exit"
					onclick={() => {
						loading = true;
						void wrapClientError(client.POST('/api/v1/shutdown'))
							.catch((e) => {
								toast({
									color: 'firebrick',
									message: `Failed to quit SISR.\n Error: ${e}`
								});
							})
							.finally(() => {
								loading = false;
							});
					}}>Exit SISR</button>
				<button
					onclick={() => {
						loading = true;
						wrapClientError(client.POST('/api/v1/enable_cef_remote_debug', { body: { restart_sisr: true } }))
							.catch((e) => {
								toast({
									color: 'firebrick',
									message: `Failed to enable Steam CEF debugging.\n Error: ${e}`
								});
							})
							.finally(() => {
								loading = false;
								invalidateAll();
							});
					}}>Enable and restart Steam</button>
			</div>
		</div>
		{#if loading}
			<div class="spinner-container" transition:fade>
				<Spinner size="10em" />
			</div>
		{/if}
	</div>
</Modal>
