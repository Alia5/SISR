<script lang="ts">
import type { components } from '$lib/api/openapi';
import Modal from '$lib/components/Modal.svelte';
import { onMount } from 'svelte';
import ConnectViiper from './connect-viiper.svelte';
import { client, wrapClientError } from '$lib/api/client';
import { toast } from '$lib/toaster/toaster.svelte';
import SISRMarkerModal from './setup-modals/SISRMarkerModal.svelte';
import CefRemoteDebugModal from './setup-modals/CefRemoteDebugModal.svelte';
import SteamNotRunningModal from './setup-modals/SteamNotRunningModal.svelte';
import InitalSetupModal from './setup-modals/InitalSetupModal.svelte';

const {
	steamStatus,
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	inputInfo
}: {
	steamStatus: components['schemas']['SteamStatus'];
	inputInfo: components['schemas']['InputInfoResponse'];
} = $props();

let connectViiper = $state<ConnectViiper>()!;

let initialLaunch = steamStatus.initial_launch;

let showInitialSetupModal = $derived(initialLaunch);

let showSteamNotRunningModal = $derived.by(() => {
	if (initialLaunch) {
		return false;
	}
	if (steamStatus.no_steam_mode) {
		return false;
	}
	return !steamStatus.running;
});
let showCefRemoteDebugDisabledModal = $derived.by(() => {
	if (initialLaunch) {
		return false;
	}
	if (steamStatus.no_steam_mode) {
		return false;
	}
	if (showSteamNotRunningModal) {
		return false;
	}
	return !steamStatus.remote_debug.enabled;
});
let showSisrMarkerNotPresentModal = $derived.by(() => {
	if (initialLaunch) {
		return false;
	}
	if (steamStatus.no_steam_mode) {
		return false;
	}
	if (steamStatus.launched_via_steam) {
		return false;
	}
	if (showCefRemoteDebugDisabledModal || showSteamNotRunningModal) {
		return false;
	}
	return !steamStatus.marker_shortcut_present && !steamStatus.launched_via_steam;
});

onMount(() => {
	if (!showSteamNotRunningModal && !showCefRemoteDebugDisabledModal && !showSisrMarkerNotPresentModal) {
		void connectViiper.connect();
	} else {
		void wrapClientError(
			client.POST('/api/v1/show_hide_ui', {
				body: {
					show: true
				}
			})
		).catch((e) => {
			toast({
				color: 'firebrick',
				message: `Failed to minimize SISR.\n Error: ${e}`
			});
		});
	}
});

let initialSetupModal = $state<Modal>()!;
let steamNotRunningModal = $state<Modal>()!;
let cefRemoteDebugDisabledModal = $state<Modal>()!;
let sisrMarkerNotPresentModal = $state<Modal>()!;
</script>

<ConnectViiper bind:this={connectViiper} />

<div style="display: contents">
	<InitalSetupModal bind:modal={initialSetupModal} bind:show={showInitialSetupModal} />
	<SteamNotRunningModal bind:modal={steamNotRunningModal} bind:show={showSteamNotRunningModal} />
	<CefRemoteDebugModal
		bind:modal={cefRemoteDebugDisabledModal}
		bind:show={showCefRemoteDebugDisabledModal} />
	<SISRMarkerModal bind:modal={sisrMarkerNotPresentModal} bind:show={showSisrMarkerNotPresentModal} />
</div>

<style lang="postcss">
div {
	& :global(.dialog-content) {
		display: grid;
		place-self: center;
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		gap: 1em;
	}

	& :global(.button-group) {
		display: flex;
		gap: 1em;
		justify-content: end;
		margin-top: 1em;
	}
	& :global(.spinner-container) {
		display: grid;
		place-items: center;
		position: absolute;
		inset: 0;
		background: rgb(0 0 0 / 0.5);
		backdrop-filter: blur(2px);
	}

	& :global(button[name='exit']) {
		background-color: rgb(128 128 128);
	}
	& :global(button) {
		padding: 1em 2em;
	}

	& :global(em) {
		color: var(--color-muted);
	}
}
</style>
