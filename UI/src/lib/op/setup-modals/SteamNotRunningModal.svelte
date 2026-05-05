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
		<h2>Steam is not running</h2>
		<div>
			<p>SISR requieres Steam running in order to redirect Steam Input to the system.</p>
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
						wrapClientError(client.POST('/api/v1/restart_steam'))
							.catch((e) => {
								toast({
									color: 'firebrick',
									message: `Failed to send Steam restart command.\n Error: ${e}`
								});
							})
							.finally(() => {
								loading = false;
								invalidateAll();
								void wrapClientError(client.POST('/api/v1/restart_sisr')).catch((e) => {
									toast({
										color: 'firebrick',
										message: `Failed to restart SISR.\n Error: ${e}`
									});
								});
							});
					}}>Attempt to start Steam and try again</button>
			</div>
		</div>
		{#if loading}
			<div class="spinner-container" transition:fade>
				<Spinner size="10em" />
			</div>
		{/if}
	</div>
</Modal>
