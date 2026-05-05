<script lang="ts" module>
export const TOAST_POSITIONS = [
	'top-left',
	'top-center',
	'top-right',
	'bottom-left',
	'bottom-center',
	'bottom-right',
	'center-left',
	'center',
	'center-right'
] as const;
</script>

<script lang="ts">
</script>

<div data-toast-container>
	{#each TOAST_POSITIONS as position (position)}
		<div data-toast-position={position} style="grid-area: {position}"></div>
	{/each}
</div>

<style lang="postcss">
[data-toast-container] {
	position: fixed;
	inset: 0;
	pointer-events: none;
	display: grid;
	grid-template-areas:
		'top-left    top-center    top-right'
		'center-left center        center-right'
		'bottom-left bottom-center bottom-right';
	gap: 1em;
	padding: 2em;
	isolation: isolate;
	z-index: 9999;
}

[data-toast-position] {
	display: grid;
	width: 100%;
	height: 100%;
	transition: grid-template-rows 300ms ease-in-out;
	& :global(> *:not(:first-child)) {
		& :global(> :first-child) {
			pointer-events: all;
			margin-top: 1em;
		}
	}
}

[data-toast-position='top-left'] {
	justify-items: start;
	align-content: start;
}
[data-toast-position='top-center'] {
	justify-items: center;
	align-content: start;
}
[data-toast-position='top-right'] {
	justify-items: end;
	align-content: start;
}
[data-toast-position='center-left'] {
	justify-items: start;
	align-content: center;
}

[data-toast-position='center'] {
	place-items: center;
}
[data-toast-position='center-right'] {
	justify-items: end;
	align-content: center;
}

[data-toast-position='bottom-left'] {
	justify-items: start;
	align-content: end;
}
[data-toast-position='bottom-center'] {
	justify-items: center;
	align-content: end;
}
[data-toast-position='bottom-right'] {
	justify-items: end;
	align-content: end;
}
</style>
