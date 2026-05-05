<script lang="ts">
import { beforeNavigate } from '$app/navigation';
import type { SwipeEndEventDetail } from '@svelte-put/swipeable';
import type { Snippet } from 'svelte';
import { fade } from 'svelte/transition';
let {
	open = $bindable(false),
	// eslint-disable-next-line no-useless-assignment
	slide_touch = $bindable(true),
	children,
	...props
}: {
	open?: boolean;
	slide_touch?: boolean;
	children?: Snippet;
	'--background'?: string;
	'--backdrop-filter'?: string;
	'--background-opacity'?: `${number}%`;
	'--background-opacity-multi'?: `${number}`;
} = $props();

// svelte-ignore state_referenced_locally
let scrim_opa_multi = $state(Number(props['--background-opacity-multi'] ?? 1));

export function show() {
	open = true;
}
export function close() {
	open = false;
}
export function toggle() {
	open = !open;
}

export function swipeend(e: CustomEvent<SwipeEndEventDetail>) {
	const { passThreshold } = e.detail;
	if (passThreshold) {
		close();
	}
}

// const onkeydown = (event: KeyboardEvent) => {
// 	if (!open) {
// 		return;
// 	}
// 	if (event.code === 'Escape') {
// 		open = !open;
// 	}
// };

export function setScrimOpacityMulti(multi: number) {
	scrim_opa_multi = multi;
}

beforeNavigate(async () => {
	open = false;
});
let HTMLDialog = $state<HTMLDialogElement>()!;
</script>

<!-- <svelte:window onkeydown={onkeydown} /> -->

{#if open}
	<dialog
		style:--background-opacity-multi={scrim_opa_multi}
		id="modal"
		bind:this={HTMLDialog}
		open
		transition:fade={{ duration: 196 }}>
		<button aria-label="modal-backdrop"></button>
		{@render children?.()}
	</dialog>
{/if}

<style lang="postcss">
dialog[open] {
	position: fixed;
	inset: 0;
	z-index: 1000;
	width: 100%;
	height: 100%;
	border: none;
	background: transparent;
	isolation: isolate;
	&::after {
		content: '';
		transition: none;
		position: absolute;
		pointer-events: none;
		inset: 0;
		z-index: -1;
		opacity: var(--background-opacity-multi, 1);
		backdrop-filter: var(--backdrop-filter, none);
		background: var(
			--background,
			linear-gradient(
				color-mix(in srgb, transparent, rgb(32, 25, 47) var(--background-opacity, 80%)),
				color-mix(in srgb, transparent, rgb(7, 4, 11) var(--background-opacity, 95%))
			)
		);
	}
}
button {
	-webkit-tap-highlight-color: transparent;
	border: none;
	border-radius: 0;
	position: absolute;
	inset: 0;
	padding: 0;
	opacity: 0;
	box-shadow: none;
	z-index: -1000;
}
</style>
