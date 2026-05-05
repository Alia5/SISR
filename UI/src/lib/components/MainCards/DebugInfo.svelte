<script lang="ts">
import type { components } from '$lib/api/openapi';
import FloatingCard from '../FloatingCard.svelte';

import IcoClose from '~icons/mdi/close';
import IcoSIAPI from '$lib/assets/siapi.svg?component';
import IcoSteam from '~icons/mdi/steam';
import IcoVIIPER from '$lib/assets/viiper_mono.svg?component';
import IcoGamepad from '~icons/fluent/xbox-controller-16-filled';

const {
	steamStatusInfo,
	inputInfo,
	onClose
}: {
	steamStatusInfo: components['schemas']['SteamStatus'];
	inputInfo: components['schemas']['InputInfoResponse'];
	onClose?: () => void;
} = $props();
</script>

<FloatingCard>
	<div id="card-content">
		<div>
			<IcoSIAPI style="width: 1.6em; height: 1.6em;" />
			<h2>Debug Info</h2>
			<button class="plain" onclick={() => onClose?.()}>
				<IcoClose style="width: 1.6em; height: 1.6em;" />
			</button>
		</div>
		<div>
			<div class="heading">
				<IcoVIIPER style="width: 1.4em; height: 1.4em;" />
				<h2>VIIPER</h2>
			</div>
			<dl>
				<dt>Address</dt>
				<dd>{inputInfo.viiper.address}</dd>
				<dt>Reachable</dt>
				<dd>{inputInfo.viiper.available ? 'Yes' : 'No'}</dd>
				<dt>Version</dt>
				<dd>{inputInfo.viiper.version ?? 'Unknown'}</dd>
				<dt>Used BusIDs</dt>
				<dd>{inputInfo.viiper.bus_ids.map((id) => id).join(', ') || 'None'}</dd>
			</dl>
			<div class="heading">
				<IcoSteam style="width: 1.4em; height: 1.4em;" />
				<h2>Steam</h2>
			</div>
			<dl>
				<dt>GameID</dt>
				<dd>{steamStatusInfo.steam_game_id}</dd>
				<dt>AppID</dt>
				<dd>{steamStatusInfo.marker_app_id}</dd>
				<dt>Launched via Steam</dt>
				<dd>{steamStatusInfo.launched_via_steam ? 'Yes' : 'No'}</dd>
				<dt>Steam Overlay</dt>
				<dd>TODO</dd>
			</dl>
			<div class="heading">
				<IcoGamepad style="width: 1.4em; height: 1.4em;" />
				<h2>Controller</h2>
			</div>
			<div>
				{#each Object.entries(inputInfo.devices) as [id, deviceInfo] (deviceInfo)}
					<h3>{deviceInfo.sdl_devices[0]?.gamepad_infos?.name}</h3>
					<dl style="padding-left: 1em;">
						<dt>ID</dt>
						<dd>{id}</dd>
						<dt>VIIPER type</dt>
						<dd>{deviceInfo.viiper_type}</dd>
						<dt>SteamHandle</dt>
						<dd>{deviceInfo.steam_handle}</dd>
						<dt>Has VIIPER Device</dt>
						<dd>{deviceInfo.has_viiper_device ? 'Yes' : 'No'}</dd>
						<dt>Corresponding Device ID</dt>
						<dd>{deviceInfo.corresponding_device_id}</dd>
						<dt>Serial</dt>
						<dd>
							{deviceInfo?.sdl_devices
								.map((d) => d?.gamepad_infos?.serial ?? d?.joystick_infos?.serial)
								?.join(', ') || 'N/A'}
						</dd>
						<dt>Path</dt>
						<dd>
							{deviceInfo?.sdl_devices
								.map((d) => d?.gamepad_infos?.path ?? d?.joystick_infos?.path)
								?.join(', ') || 'N/A'}
						</dd>
					</dl>
				{/each}
				{#if Object.keys(inputInfo.devices).length === 0}
					<p>No controllers connected</p>
				{/if}
			</div>
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
		gap: 1em;
		& button {
			padding: 0.5em;
		}
	}
	& > :last-child {
		overflow: auto;
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

dl {
	display: grid;
	grid-template-columns: min-content auto;
	column-gap: 1em;
	row-gap: 0.25em;
	padding-bottom: 1.5em;
	dt {
		font-weight: bold;
		white-space: nowrap;
	}
	dd {
		color: var(--text-muted);
	}
}
</style>
