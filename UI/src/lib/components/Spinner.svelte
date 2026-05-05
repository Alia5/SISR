<!-- Stolen (but improved) from hhttps://cssloaders.github.io -->
<script lang="ts">
const {
	style,
	thickness = '0.3em',
	size
} = $props<{
	style?: string;
	size?: string;
	thickness?: string;
}>();
</script>

<div style={`${style}; --thickness: ${thickness}; ${size ? `--height: ${size}` : ''}`}></div>

<style lang="postcss">
div {
	aspect-ratio: 1 / 1;
	height: var(--height, 100%);
	width: var(--height, 100%);
	border-radius: 50%;
	overflow: hidden;
	display: inline-block;
	position: relative;
	border: var(--thickness) solid;
	border-color: var(--text-color) var(--text-color) transparent transparent;
	box-sizing: border-box;
	animation: rotation 1s linear infinite;
}
div::after,
div::before {
	content: '';
	box-sizing: border-box;
	position: absolute;
	left: 0;
	right: 0;
	top: 0;
	bottom: 0;
	margin: auto;
	border: var(--thickness) solid;
	border-color: transparent transparent var(--color-primary) var(--color-primary);
	aspect-ratio: 1;
	height: 90%;
	border-radius: 50%;
	box-sizing: border-box;
	animation: rotationBack 0.5s linear infinite;
	transform-origin: center center;
}
div::before {
	aspect-ratio: 1;
	height: 80%;
	border-color: var(--text-color) var(--text-color) transparent transparent;
	animation: rotation 1.5s linear infinite;
}

@keyframes rotation {
	0% {
		transform: rotate(0deg);
	}
	100% {
		transform: rotate(360deg);
	}
}
@keyframes rotationBack {
	0% {
		transform: rotate(0deg);
	}
	100% {
		transform: rotate(-360deg);
	}
}
</style>
