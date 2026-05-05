<script lang="ts">
import { page } from '$app/state';
import { client, wrapClientError } from '$lib/api/client';
import { tooltip } from '$lib/attachments/tooltip.svelte';
import { statusCodeNames } from '$lib/statuscodes';
import { toast } from '$lib/toaster/toaster.svelte';
import { onMount } from 'svelte';

let eyes = $state<{
	left: HTMLElement;
	right: HTMLElement;
}>({
	left: undefined!,
	right: undefined!
})!;

onMount(() => {
	const group = document.querySelector('#sc2>*>svg>g>:last-child')!;
	eyes.left = group.children[0] as HTMLElement;
	eyes.right = group.children[1] as HTMLElement;
});
</script>

<main>
	<div>
		<h1>{page.status}</h1>
		{#if statusCodeNames[page.status] && statusCodeNames[page.status] !== page.error?.message}
			<h3>{statusCodeNames[page.status]}</h3>
		{/if}
		<h2>{page.error?.message}</h2>
		<p>An error occured, please try to restart the app</p>
	</div>
	<button
		class="quit"
		{@attach tooltip({
			arrow: true,
			arrowFollowCursor: true,
			content: 'Completely shut down SISR'
		})}
		onclick={() =>
			void wrapClientError(client.POST('/api/v1/shutdown')).catch((e) => {
				toast({
					color: 'firebrick',
					message: `Failed to quit SISR.\n Error: ${e}`
				});
			})}>Quit SISR</button>
</main>

<style lang="postcss">
main {
	display: grid;
	place-items: center;
}

div {
	display: grid;
	place-items: center;
	width: 100%;
}

h1 {
	font-size: 6em;
	font-weight: bold;
}

h2 {
	color: var(--highlight-color);
	font-size: 2em;
}

h3 {
	translate: 0 -1em;
}

.quit {
	position: absolute;
	bottom: 0;
	right: 0;
	border-radius: 1em 0 0 0;
	display: grid;
	place-items: center;
	padding: 1em 2em !important;
	font-weight: bold;
}
</style>
