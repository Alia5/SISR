<script lang="ts">
import type { components } from '$lib/api/openapi';
import FloatingCard from '../FloatingCard.svelte';

import IcoClose from '~icons/mdi/close';
import IcoGamepad from '~icons/fluent/xbox-controller-16-filled';
import IcoSettings from '~icons/mdi/cog';
import { client } from '$lib/api/client';
import { log } from '$lib/log';
import { toast } from '$lib/toaster/toaster.svelte';

const STEAM_DESKTOP_CONFIG_APPID = 413080;

const {
	steamStatusInfo,
	inputInfo,
	onClose
}: {
	steamStatusInfo: components['schemas']['SteamStatus'];
	inputInfo: components['schemas']['InputInfoResponse'];
	onClose?: () => void;
} = $props();

let allowDesktopConfig = $derived(!inputInfo.force_controller_config);
</script>

<FloatingCard>
	<div id="card-content">
		<div>
			<IcoSettings style="width: 1.6em; height: 1.6em;" />
			<h2>Quick Settings</h2>
			<button class="plain" onclick={() => onClose?.()}>
				<IcoClose style="width: 1.6em; height: 1.6em;" />
			</button>
		</div>
		<div>
			{#if !steamStatusInfo.no_steam_mode}
				<div class="checkbox-wrap">
					<label for="allow-desktop-config"> Allow Desktop config </label>
					<input
						type="checkbox"
						id="allow-desktop-config"
						name="allow-desktop-config"
						onchange={() => {
							void client
								.POST('/api/v1/force_controller_config', {
									body: {
										enforce: !allowDesktopConfig
									}
								})
								.catch((e) => {
									log.error('Failed to update controller config setting', 'error', e);
									toast({
										color: 'firebrick',
										message: 'Failed to change allow desktop config'
									});
								});
						}}
						bind:checked={allowDesktopConfig} />
				</div>
				<button
					onclick={() => {
						void client
							.POST('/api/v1/open_steam_controller_config', {
								body: {
									app_id: allowDesktopConfig
										? STEAM_DESKTOP_CONFIG_APPID
										: steamStatusInfo.marker_app_id
								}
							})
							.catch((e) => {
								log.error('Failed to open Steam Input Layout configurator', 'error', e);
								toast({
									color: 'firebrick',
									message: 'Failed to open Steam Input Layout configurator'
								});
							});
					}}
					><IcoGamepad style="width: 1.4em; height: 1.4em;" />
					<div>
						<span>Open Steam Input Layout configurator</span>
						{#if allowDesktopConfig}
							(Desktop Layout)
						{/if}
					</div></button>
			{/if}
		</div>
	</div>
</FloatingCard>

<style lang="postcss">
#card-content {
	display: flex;
	flex-direction: column;
	gap: 1em;
	height: 100%;

	& > :first-child {
		display: grid;
		grid-template-columns: min-content 1fr min-content;
		justify-content: center;
		align-items: center;
		gap: 1em;
		& button {
			padding: 0.5em;
		}
	}
	& > :last-child {
		overflow: auto;
		display: grid;
		gap: 1em;
		padding: 1em;
		width: 100%;
	}
}

h3 {
	margin-top: 1em;
	margin-bottom: 0.5em;
}

.heading {
	display: grid;
	grid-template-columns: min-content min-content;
	gap: 0.5em;
	place-items: center;
	place-self: center;
}

.checkbox-wrap {
	display: grid;
	grid-auto-flow: column;
	gap: 0.5em;
	align-items: center;
	justify-content: start;
}

button {
	display: grid;
	grid-auto-flow: column;
	gap: 0.5em;
	place-items: center;
	width: fit-content;
	& div {
		display: grid;
		place-items: center;
		gap: 0.25em;
	}
}
</style>
