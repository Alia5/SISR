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
		<h2>Initial Setup</h2>
		<div>
			<p>It appears this is the first time you are running SISR.</p>
			<br />
			<p>SISR requires some initial setup to work properly.</p>
			<p>
				This requires enabling a developer/debug interface in Steam as well as adding a marker
				shortcut to Steam.
			</p>
			<p>
				The <i>"SISR Marker"</i> is a special shortcut whose Steam Input layout will be used when SISR is
				launched outside of Steam
			</p>
			<br />
			<p>Steam will be restarted as part of the setup process.</p>
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
					onclick={async () => {
						loading = true;
						try {
							await wrapClientError(client.POST('/api/v1/enable_cef_remote_debug'));
						} catch (e) {
							loading = false;
							toast({
								color: 'firebrick',
								message: `Failed to enable CEF remote debug.\n Error: ${e}`
							});
							return;
						}
						try {
							await wrapClientError(client.POST('/api/v1/create_marker_shortcut'));
						} catch (e) {
							loading = false;
							toast({
								color: 'firebrick',
								message: `Failed to add marker shortcut.\n Error: ${e}`
							});
							return;
						}
						localStorage.setItem('initial_setup_complete', 'true');
						loading = false;
						invalidateAll();
						void wrapClientError(client.POST('/api/v1/restart_sisr')).catch((e) => {
							toast({
								color: 'firebrick',
								message: `Failed to restart SISR.\n Error: ${e}`
							});
						});
					}}>Setup Now</button>
			</div>
		</div>
		{#if loading}
			<div class="spinner-container" transition:fade>
				<Spinner size="10em" />
			</div>
		{/if}
	</div>
</Modal>
