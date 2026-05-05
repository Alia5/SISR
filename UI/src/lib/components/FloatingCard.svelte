<script module lang="ts">
let z = 0;
</script>

<script lang="ts">
import type { Snippet } from 'svelte';

const interactiveSelector = [
	'a',
	'button',
	'input',
	'select',
	'textarea',
	'label',
	'summary',
	'option',
	'[draggable="true"]',
	'[contenteditable]:not([contenteditable="false"])',
	'[tabindex]:not([tabindex="-1"])',
	'[role="button"]',
	'[role="link"]',
	'[role="checkbox"]',
	'[role="radio"]',
	'[role="switch"]',
	'[role="slider"]',
	'[role="spinbutton"]',
	'[role="textbox"]'
].join(',');

const {
	children
}: {
	children?: Snippet;
} = $props();

let card = $state<HTMLElement>()!;
let zIndex = $state(++z);
let drag = $state({
	isDragging: false,
	pointerId: undefined as number | undefined,
	bodyUserSelect: undefined as string | undefined,
	pointerStart: { x: 0, y: 0 },
	translate: { x: 0, y: 0 },
	startTranslate: { x: 0, y: 0 },
	bounds: {
		minX: 0,
		maxX: 0,
		minY: 0,
		maxY: 0
	}
});

let isInteractiveTarget = (target: EventTarget | null) => {
	if (target instanceof Element) {
		return !!target.closest(interactiveSelector);
	}

	if (target instanceof Node) {
		return !!target.parentElement?.closest(interactiveSelector);
	}

	return false;
};

let onPointerDown = (event: PointerEvent) => {
	zIndex = ++z;

	if (drag.isDragging || isInteractiveTarget(event.target)) {
		return;
	}

	const cardRect = card.getBoundingClientRect();
	const bodyRect = document.body.getBoundingClientRect();

	drag.pointerId = event.pointerId;
	drag.isDragging = true;
	drag.bodyUserSelect = document.body.style.userSelect;
	drag.pointerStart = {
		x: event.clientX,
		y: event.clientY
	};
	drag.startTranslate = {
		x: drag.translate.x,
		y: drag.translate.y
	};
	drag.bounds = {
		minX: bodyRect.left - cardRect.left + drag.translate.x,
		maxX: bodyRect.right - cardRect.right + drag.translate.x,
		minY: bodyRect.top - cardRect.top + drag.translate.y,
		maxY: bodyRect.bottom - cardRect.bottom + drag.translate.y
	};

	document.body.style.userSelect = 'none';

	card.setPointerCapture(event.pointerId);
};

let onPointerMove = (event: PointerEvent) => {
	if (!drag.isDragging || drag.pointerId !== event.pointerId) {
		return;
	}

	drag.translate = {
		x: Math.min(
			Math.max(drag.startTranslate.x + (event.clientX - drag.pointerStart.x), drag.bounds.minX),
			drag.bounds.maxX
		),
		y: Math.min(
			Math.max(drag.startTranslate.y + (event.clientY - drag.pointerStart.y), drag.bounds.minY),
			drag.bounds.maxY
		)
	};
};

let onPointerUp = (event: PointerEvent) => {
	if (drag.pointerId !== event.pointerId) {
		return;
	}

	document.body.style.userSelect = drag.bodyUserSelect ?? '';
	drag.isDragging = false;
	drag.pointerId = undefined;
	drag.bodyUserSelect = undefined;
};

let onLostPointerCapture = () => {
	document.body.style.userSelect = drag.bodyUserSelect ?? '';
	drag.isDragging = false;
	drag.pointerId = undefined;
	drag.bodyUserSelect = undefined;
};

let size = $state({
	width: undefined as number | undefined,
	height: undefined as number | undefined
});
let detached = $state(false);
let detachedPos = $state({
	left: 0,
	top: 0
});

let resize = $state({
	isResizing: false,
	edge: undefined as 'right' | 'bottom' | 'corner' | undefined,
	pointerId: undefined as number | undefined,
	bodyUserSelect: undefined as string | undefined,
	pointerStart: {
		x: 0,
		y: 0
	},
	startSize: {
		width: 0,
		height: 0
	},
	maxSize: {
		width: 0,
		height: 0
	}
});

let onResizePointerDown = (event: PointerEvent, edge: 'right' | 'bottom' | 'corner') => {
	if (resize.isResizing || drag.isDragging) {
		return;
	}
	event.stopPropagation();
	zIndex = ++z;
	const cardRect = card.getBoundingClientRect();
	if (!detached) {
		const parentRect = card.offsetParent?.getBoundingClientRect() ?? { left: 0, top: 0 };
		detachedPos = {
			left: cardRect.left - parentRect.left,
			top: cardRect.top - parentRect.top
		};
		drag.translate = { x: 0, y: 0 };
		detached = true;
	}
	const bodyRect = document.body.getBoundingClientRect();
	resize.isResizing = true;
	resize.edge = edge;
	resize.pointerId = event.pointerId;
	resize.bodyUserSelect = document.body.style.userSelect;
	resize.pointerStart = { x: event.clientX, y: event.clientY };
	resize.startSize = { width: cardRect.width, height: cardRect.height };
	resize.maxSize = {
		width: bodyRect.right - cardRect.left,
		height: bodyRect.bottom - cardRect.top
	};
	document.body.style.userSelect = 'none';
	(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
};

let onResizePointerMove = (event: PointerEvent) => {
	if (!resize.isResizing || resize.pointerId !== event.pointerId) {
		return;
	}
	const dx = event.clientX - resize.pointerStart.x;
	const dy = event.clientY - resize.pointerStart.y;
	if (resize.edge === 'right' || resize.edge === 'corner') {
		size.width = Math.min(Math.max(resize.startSize.width + dx, 140), resize.maxSize.width);
	}
	if (resize.edge === 'bottom' || resize.edge === 'corner') {
		size.height = Math.min(Math.max(resize.startSize.height + dy, 80), resize.maxSize.height);
	}
};

let onResizeEnd = (event: PointerEvent) => {
	if (resize.pointerId !== event.pointerId) {
		return;
	}
	document.body.style.userSelect = resize.bodyUserSelect ?? '';
	resize.isResizing = false;
	resize.edge = undefined;
	resize.pointerId = undefined;
	resize.bodyUserSelect = undefined;
};

let onResizeLostCapture = () => {
	document.body.style.userSelect = resize.bodyUserSelect ?? '';
	resize.isResizing = false;
	resize.edge = undefined;
	resize.pointerId = undefined;
	resize.bodyUserSelect = undefined;
};
</script>

<section
	role="tab"
	tabindex="-1"
	bind:this={card}
	style:transform={`translate(${drag.translate.x}px, ${drag.translate.y}px)`}
	style:position={detached ? 'absolute' : undefined}
	style:left={detached ? `${detachedPos.left}px` : undefined}
	style:top={detached ? `${detachedPos.top}px` : undefined}
	style:width={size.width ? `${size.width}px` : undefined}
	style:height={size.height ? `${size.height}px` : undefined}
	style:z-index={zIndex}
	style:cursor={drag.isDragging ? 'grabbing' : undefined}
	onpointerdown={onPointerDown}
	onpointermove={onPointerMove}
	onpointerup={onPointerUp}
	onpointercancel={onLostPointerCapture}
	onlostpointercapture={onLostPointerCapture}>
	<div class="card glass">
		{@render children?.()}
	</div>
	<div
		aria-hidden="true"
		onpointerdown={(e) => onResizePointerDown(e, 'right')}
		onpointermove={onResizePointerMove}
		onpointerup={onResizeEnd}
		onpointercancel={onResizeLostCapture}
		onlostpointercapture={onResizeLostCapture}>
	</div>
	<div
		aria-hidden="true"
		onpointerdown={(e) => onResizePointerDown(e, 'bottom')}
		onpointermove={onResizePointerMove}
		onpointerup={onResizeEnd}
		onpointercancel={onResizeLostCapture}
		onlostpointercapture={onResizeLostCapture}>
	</div>
	<div
		aria-hidden="true"
		onpointerdown={(e) => onResizePointerDown(e, 'corner')}
		onpointermove={onResizePointerMove}
		onpointerup={onResizeEnd}
		onpointercancel={onResizeLostCapture}
		onlostpointercapture={onResizeLostCapture}>
	</div>
</section>

<style lang="postcss">
section {
	touch-action: none;
	cursor: default;
	position: relative;
	padding: 0;
	min-width: 36ch;
	height: 100%;

	&:active {
		cursor: grabbing;
	}

	& > :first-child {
		min-height: 100%;
		height: 100%;
	}

	& > :nth-last-child(3) {
		height: 100%;
		right: 0;
		top: 0;
		width: 4px;
		background: firebrick;
		position: absolute;
		padding: 0;
		cursor: ew-resize;
		opacity: 0;
	}
	& > :nth-last-child(2) {
		height: 4px;
		bottom: 0;
		width: 100%;
		background: firebrick;
		position: absolute;
		padding: 0;
		cursor: ns-resize;
		opacity: 0;
	}
	& > :last-child {
		height: 6px;
		width: 6px;
		bottom: 0;
		right: 0;
		background: rebeccapurple;
		position: absolute;
		padding: 0;
		cursor: nwse-resize;
		opacity: 0;
	}
}
</style>
