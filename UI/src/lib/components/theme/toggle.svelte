<script lang="ts" module>
export const themeSelection = $state({
	darkMode: undefined as boolean | undefined
});
</script>

<script lang="ts">
import { page } from '$app/state';
import { onMount } from 'svelte';

import PhMoon from '~icons/ph/moon';
import PhSun from '~icons/ph/sun';

if (themeSelection.darkMode === undefined) {
	themeSelection.darkMode = page.data.theme === 'dark';
}

const toggleTheme = () => {
	const html = document.documentElement;
	const wrap = () => {
		// no-op, just for the transition
		if (themeSelection.darkMode) {
			html.style.colorScheme = 'light';
			document.documentElement.setAttribute('data-theme', 'light');
			document.cookie = 'theme=light;path=/;max-age=31536000';
		} else {
			html.style.colorScheme = 'dark';
			document.documentElement.setAttribute('data-theme', 'dark');
			document.cookie = 'theme=dark;path=/;max-age=31536000';
		}
		themeSelection.darkMode = !themeSelection.darkMode;
	};
	wrap();

	// if (!document.startViewTransition) {
	// 	wrap();
	// 	return;
	// }
	// try {
	// 	// for firefox it'd be better to not use a view-transition here, but oh well...
	// 	// document.startViewTransition({
	// 	// 	types: ['theme'],
	// 	// 	update: wrap
	// 	// });
	// 	document.startViewTransition(wrap);
	// } catch {
	// 	wrap();
	// }
};

onMount(() => {
	const savedTheme = themeSelection.darkMode! ? 'dark' : 'light';
	const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
	const isDark = savedTheme === 'dark' || (!savedTheme && prefersDark);

	themeSelection.darkMode = isDark;

	if (isDark && !savedTheme) {
		document.cookie = 'theme=dark;path=/;max-age=31536000';
	} else if (!isDark && !savedTheme) {
		document.cookie = 'theme=light;path=/;max-age=31536000';
	}
});
</script>

<div class="wrapper">
	<input
		name="Theme toggle"
		type="checkbox"
		class="toggle"
		checked={themeSelection.darkMode}
		onchange={() => toggleTheme()}
		aria-label="Theme Toggle" />
	<div class="icon {themeSelection.darkMode ? 'checked' : ''}">
		{#if themeSelection.darkMode}
			<!-- <Icon icon="ph:moon" width="1.2em" height="1.2em" /> -->
			<PhMoon />
		{:else}
			<!-- <Icon icon="ph:sun" width="1.2em" height="1.2em" /> -->
			<PhSun />
		{/if}
	</div>
</div>

<style lang="postcss">
.wrapper {
	position: relative;
}

.icon {
	--toggle-height: 1.75em;
	--toggle-width: 3em;
	--indicator-padding: 0.5em;
	width: auto;
	height: var(--toggle-height);
	position: absolute;
	top: 0;
	bottom: 0;
	display: grid;
	align-items: center;
	pointer-events: none;
	left: calc(var(--indicator-padding) / 2);
	&.checked {
		left: calc(var(--toggle-width) - var(--toggle-height) + (var(--indicator-padding) / 2));
	}
	& :global(svg) {
		padding: 0.1em;
	}
	transition: left var(--transition-duration) var(--default-ease);
}

input[type='checkbox']:is(.toggle) {
	--toggle-height: 1.75em;
	--toggle-width: 3em;
	--indicator-padding: 0.5em;

	position: relative;

	&::before {
		background-color: var(--background-neutral-alpha);
	}
	&::after {
		content: '';
		background-color: var(--inverse-text-color);
	}
	&:checked {
		&::after {
			content: '';
			left: calc(var(--toggle-width) - var(--toggle-height) + (var(--indicator-padding) / 2));
		}
	}
}
</style>
