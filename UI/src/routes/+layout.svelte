<script lang="ts">
import { invalidateAll, onNavigate } from '$app/navigation';
import favicon from '$lib/assets/favicon.svg?url';

import 'unfonts.css';
import { links } from 'unplugin-fonts/head';
import '../css/main.pcss';
const { children } = $props();

window.invalidateAll = invalidateAll;

onNavigate((navigation) => {
	if (!document.startViewTransition) {
		return;
	}

	// prevent view transition for same-page navigations,
	// there should not be a fucking transition if nothing changes... 🙄
	if (navigation.from?.url.pathname === navigation.to?.url.pathname) {
		return;
	}

	return new Promise((resolve) => {
		document.startViewTransition(async () => {
			resolve();
			await navigation.complete;
		});
	});
});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	{#each links as link (link?.attrs?.href)}
		{#if link?.attrs?.onload}
			<link
				{...link?.attrs || {}}
				onload={function () {
					this.rel = 'stylesheet';
				}} />
		{:else}
			<link {...link?.attrs || {}} />
		{/if}
	{/each}
	<style>
	body,
	main,
	header,
	footer {
		transition: all var(--transition-duration) var(--default-ease);
	}
	</style>
</svelte:head>

{@render children()}

<style lang="postcss">
:global(body) {
	display: grid;
	grid-template-rows: 1fr;
	min-height: 100svh;
	max-width: 100dvw;
}

:global(main) {
	grid-row: 1 / span 1;
	grid-column: 1 / span 1;
}
</style>
