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
		<h2>SISR Marker</h2>
		<div>
			<p>
				SISR uses a <i>marker shortcut</i> added to Steam and uses its Steam Input Layout when not launched
				via Steam
			</p>
			<p>Should the marker shortcut be added to Steam now?</p>
			<br />
			<p>Otherwise add SISR as a non-Steam-Shortcut and launch SISR from Steam</p>
			<p><em>(which you can do at anytime, regardless of the marker shortcut)</em></p>
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
						wrapClientError(client.POST('/api/v1/create_marker_shortcut'))
							.catch((e) => {
								toast({
									color: 'firebrick',
									message: `Failed to add marker shortcut.\n Error: ${e}`
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
					}}>Create</button>
			</div>
		</div>
		{#if loading}
			<div class="spinner-container" transition:fade>
				<Spinner size="10em" />
			</div>
		{/if}
	</div>
</Modal>
