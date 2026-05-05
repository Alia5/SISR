<script lang="ts" module>
import { log } from '$lib/log';
import { mount, unmount, type Snippet } from 'svelte';
import { quadIn, quadOut } from 'svelte/easing';
import { slide } from 'svelte/transition';
import type { TOAST_POSITIONS } from './container.svelte';
import ToastContainer from './container.svelte';
import Toast from './toaster.svelte';

export const ensureContainer = () => {
	const exists = !!document.querySelector('[data-toast-container]');
	if (exists) {
		return;
	}
	mount(ToastContainer, {
		target: document.body,
		intro: false
	});
};

export const toast = ({
	message,
	snippet,
	duration = 3666,
	transitionDuration = 196,
	position = 'top-right',
	color
}: {
	message?: string;

	snippet?: Snippet;
	duration?: number;
	transitionDuration?: number;
	position?: (typeof TOAST_POSITIONS)[number];
	offset?: number | string | { x: number | string; y: number | string };
	color: string;
}) => {
	ensureContainer();

	let usedSnippet = snippet;
	if (message && !snippet) {
		usedSnippet = defaultSnippet as Snippet;
	}
	let snippetMounter: ReturnType<typeof mount> | undefined;

	const targetSelector = `[data-toast-position="${position}"]`;

	const target = document.querySelector(targetSelector);
	if (!target) {
		log.error('Could not find toast target!');
		return;
	}
	snippetMounter = mount(Toast, {
		target: target,
		intro: true,
		props: {
			children: usedSnippet,
			position,
			color,
			message,
			inDelay: 0,
			transitionDuration
		}
	});
	const remove = () => {
		if (snippetMounter) {
			unmount(snippetMounter, { outro: true });
			snippetMounter = undefined;
		}
	};
	setTimeout(remove, duration);
};
</script>

<script lang="ts">
let {
	children,
	...rest
}: {
	children?: Snippet<[Record<string, unknown>]>;
} = $props();
</script>

<div data-toast-snippet-mounter style="display: contents;">
	{#if children}
		{@render children?.({ ...rest })}
	{/if}
</div>

{#snippet defaultSnippet({
	message = '',
	inDelay = 0,
	transitionDuration = 196,
	color = 'transparent'
}: {
	message?: string;
	inDelay?: number;
	transitionDuration?: number;
	color?: string;
})}
	<div
		role="alert"
		style="--color: {color}"
		in:slide|global={{ duration: transitionDuration, delay: inDelay, easing: quadOut }}
		out:slide|global={{ duration: transitionDuration, easing: quadIn }}>
		<p>{message}</p>
	</div>
{/snippet}

<style lang="postcss">
:global([role='alert']) {
	padding: 1em;
	position: relative;
	isolation: isolate;
	background: var(--card-glass);
	background-color: var(--color, transparent);
	border-radius: var(--border-radius);
	box-shadow: var(--card-shadow);
	&::before {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: inherit;
		border: 1px solid transparent;
		background: var(--card-border-pseudo-gradient) border-box;
		mask:
			linear-gradient(black, black) border-box,
			linear-gradient(black, black) padding-box;
		mask-composite: subtract;
		opacity: 0.5;
		z-index: -1;
	}
}
</style>
