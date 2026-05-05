<script lang="ts" module>
import {
	arrow,
	autoPlacement as autoPlacementMiddleware,
	autoUpdate,
	computePosition,
	flip as flipMiddleware,
	offset as offsetMiddleware,
	shift as shiftMiddleware,
	type ArrowOptions,
	type AutoPlacementOptions,
	type Middleware,
	type OffsetOptions,
	type Placement,
	type ShiftOptions
} from '@floating-ui/dom';
import { mount, unmount, type Snippet } from 'svelte';
import type { Attachment } from 'svelte/attachments';
import Tooltip from './tooltip.svelte';

export type OptionalTooltipOptions = {
	placement?: Placement;
	flip?: boolean;
	autoPlacement?: boolean | AutoPlacementOptions;
	shift?: boolean | ShiftOptions;
	offset?: boolean | OffsetOptions;
	mountPoint?: 'parent' | 'trigger' | 'body';
	inDelay?: number;
	outDelay?: number;
	transitionDuration?: number;
	interactable?: boolean;
	arrow?: boolean | ArrowOptions;
	arrowFollowCursor?: boolean;
};

export type DefaultTooltipOptions = {
	content: string;
	background?: boolean | string;
};

export type SnippetTooltipOptions = {
	snippet: Snippet;
	snippetInDefaultBackground?: boolean;
};
export type TooltipOptions = OptionalTooltipOptions & (DefaultTooltipOptions | SnippetTooltipOptions);

/**
 * Create a tooltip attachment with `{@attach ...}`
 *
 * This returns a Svelte `Attachment` that you can apply to any element to show a tooltip
 * on hover/focus.
 *
 * @example
 * <button
 *   {@attach tooltip({
 *     content: 'Hello!',
 *     placement: 'top',
 *     arrow: true,
 *     outDelay: 150
 *   })}
 * >Hover me</button>
 *
 * @example
 * <!-- Custom markup via snippet -->
 * {@snippet tip()}
 *   <div class="card">Interactive <a href="/docs">docs</a></div>
 * {/snippet}
 *
 * <span {@attach tooltip({ snippet: tip, interactable: true, arrow: true })}>?</span>
 *
 * If you use a custom snippet, transitions and animations that you define on your snippet will work, too!
 * Alternativly, if a simple fade works fine for you, you can set the "snippetInDefaultBackground"
 *
 *
 * @param options
 * Tooltip configuration.
 * - `content`: string tooltip text (uses the default tooltip snippet)
 * - `snippet`: custom tooltip content
 * - `placement` / `flip` / `shift` / `offset` / `autoPlacement`: Floating UI placement behavior
 * - `arrow`: render an arrow (boolean or ArrowOptions)
 * - `arrowFollowCursor`: make the arrow track the mouse position while over the trigger
 * - `interactable`: keep tooltip open while hovering/focusing the tooltip itself
 * - `inDelay` / `outDelay`: open/close delays in ms
 * - `mountPoint`: where to mount (`'body' | 'parent' | 'trigger'`)
 *
 * @returns An `Attachment` function. Svelte calls it with the trigger element and uses the
 * returned cleanup function to remove listeners and unmount the tooltip.
 */
export const tooltip = ({
	flip = true,
	shift = true,
	offset = true,
	placement = 'top',
	autoPlacement = { allowedPlacements: ['left', 'right', 'top', 'bottom'] },
	transitionDuration = 196,
	interactable = true,
	arrowFollowCursor = false,
	...options
}: TooltipOptions): Attachment => {
	let snippet = (options as unknown as Record<string, Snippet | undefined>).snippet;
	if ((options as unknown as Record<string, unknown>).content && !snippet) {
		snippet = defaultTooltip as Snippet;
	}

	return (triggerElement: Element) => {
		const triggerEl = triggerElement as HTMLElement;
		let snippetMounter: ReturnType<typeof mount> | undefined;
		let floatingCleanup: ReturnType<typeof autoUpdate> | undefined;
		let removeTimeout: ReturnType<typeof setTimeout> | undefined;
		let tooltipElement: HTMLElement | undefined;
		let cleanupTooltipListeners: (() => void) | undefined;
		let isPointerOverTooltip = false;
		let isPointerOverTrigger = false;
		let lastPointerClient: { x: number; y: number } | undefined;
		let updatePositionRef: (() => void) | undefined;

		const normalizePadding = (padding: ArrowOptions['padding'] | undefined): number => {
			if (!padding) {
				return 0;
			}
			if (typeof padding === 'number') {
				return padding;
			}
			return Math.max(padding.top ?? 0, padding.right ?? 0, padding.bottom ?? 0, padding.left ?? 0);
		};

		const cancelRemove = () => {
			if (removeTimeout) {
				clearTimeout(removeTimeout);
				removeTimeout = undefined;
			}
		};

		const isFocusWithin = (el?: Element) => {
			if (!el) {
				return false;
			}
			const active = document.activeElement;
			if (!active) {
				return false;
			}
			return el.contains(active);
		};

		const scheduleRemove = () => {
			if (removeTimeout) {
				return;
			}
			if (interactable) {
				if (
					isPointerOverTrigger ||
					isPointerOverTooltip ||
					isFocusWithin(triggerElement) ||
					isFocusWithin(tooltipElement)
				) {
					return;
				}
			}
			if (options.outDelay) {
				removeTimeout = setTimeout(remove, options.outDelay);
			} else {
				remove();
			}
		};

		const onFocus = () => {
			cancelRemove();
			if (snippetMounter) {
				return;
			}
			const randomId = Math.floor(Math.random() * (99999 + 1)).toString();
			const snippetMounterQuerySelector = `[data-snippet-mounter][data-tooltip-trigger-id="${randomId}"]`;
			snippetMounter = mount(Tooltip, {
				target:
					options.mountPoint === 'trigger'
						? triggerElement
						: options.mountPoint === 'parent'
							? (triggerElement.parentElement ?? document.body)
							: document.body,
				intro: true,
				props: {
					children: snippet,
					tooltipTriggerId: randomId,
					flip,
					shift,
					offset,
					transitionDuration,
					interactable,
					...options
				}
			});

			const snippetMounterElement = document.querySelector(
				snippetMounterQuerySelector
			) as HTMLElement | null;
			tooltipElement = (snippetMounterElement?.firstElementChild as HTMLElement | null) ?? undefined;
			if (!tooltipElement) {
				remove();
				return;
			}
			const arrowElement = tooltipElement.querySelector('[data-tooltip-arrow]') as HTMLElement;

			if (interactable) {
				const onTooltipPointerEnter = () => {
					isPointerOverTooltip = true;
					cancelRemove();
				};
				const onTooltipPointerLeave = () => {
					isPointerOverTooltip = false;
					scheduleRemove();
				};
				const onTooltipFocusIn = () => {
					cancelRemove();
				};
				const onTooltipFocusOut = () => {
					scheduleRemove();
				};

				tooltipElement.addEventListener('pointerenter', onTooltipPointerEnter);
				tooltipElement.addEventListener('pointerleave', onTooltipPointerLeave);
				tooltipElement.addEventListener('focusin', onTooltipFocusIn);
				tooltipElement.addEventListener('focusout', onTooltipFocusOut);

				cleanupTooltipListeners = () => {
					tooltipElement?.removeEventListener('pointerenter', onTooltipPointerEnter);
					tooltipElement?.removeEventListener('pointerleave', onTooltipPointerLeave);
					tooltipElement?.removeEventListener('focusin', onTooltipFocusIn);
					tooltipElement?.removeEventListener('focusout', onTooltipFocusOut);
				};
			}

			const middleWares: Middleware[] = [inline()];
			if (autoPlacement) {
				const options =
					typeof autoPlacement === 'boolean'
						? ({
								allowedPlacements: ['top', 'bottom', 'right', 'left']
							} satisfies AutoPlacementOptions)
						: autoPlacement;
				middleWares.push(
					autoPlacementMiddleware({
						...options
					})
				);
			} else if (flip) {
				middleWares.push(flipMiddleware());
			}
			if (shift) {
				const options = typeof shift === 'boolean' ? { padding: 8 } : shift;
				middleWares.push(shiftMiddleware(options));
			}
			if (offset) {
				const options = typeof offset === 'boolean' ? { mainAxis: 8 } : offset;
				middleWares.push(offsetMiddleware(options));
			}
			if ((options as Record<string, unknown>).arrow && arrowElement) {
				const arrowOptions =
					typeof (options as Record<string, unknown>).arrow === 'boolean'
						? { element: arrowElement }
						: {
								element: arrowElement,
								padding: ((options as Record<string, unknown>)?.arrow as ArrowOptions)
									?.padding
							};
				middleWares.push(arrow(arrowOptions));
			}

			const chosenPlacement = (Array.isArray(placement) ? placement[0] : placement) ?? 'top';

			const updatePosition = () => {
				if (!triggerElement || !tooltipElement) {
					return;
				}
				computePosition(triggerElement, tooltipElement, {
					placement: chosenPlacement,
					middleware: middleWares
				}).then(({ x, y, placement, middlewareData }) => {
					Object.assign(tooltipElement?.style || {}, {
						left: `${x}px`,
						top: `${y}px`
					});
					const { x: arrowX, y: arrowY } = middlewareData.arrow ?? {};
					const staticSide = {
						top: 'bottom',
						right: 'left',
						bottom: 'top',
						left: 'right'
					}[placement.split('-')![0]!]!;
					const side = placement.split('-')![0]!;

					if (arrowElement) {
						let left = arrowX != null ? `${arrowX}px` : '';
						let top = arrowY != null ? `${arrowY}px` : '';

						const arrowFollowsCursor = arrowFollowCursor ?? false;
						if (arrowFollowsCursor && lastPointerClient && tooltipElement) {
							const tooltipRect = tooltipElement.getBoundingClientRect();
							const arrowWidth = arrowElement.offsetWidth || 0;
							const arrowHeight = arrowElement.offsetHeight || 0;

							const arrowPadding =
								typeof (options as Record<string, unknown>).arrow === 'object'
									? normalizePadding(
											((options as Record<string, unknown>).arrow as ArrowOptions)
												?.padding
										)
									: 0;

							if ((side === 'top' || side === 'bottom') && tooltipRect.width > 0) {
								const desiredCenterX = lastPointerClient.x - tooltipRect.left;
								let desiredLeft = desiredCenterX - arrowWidth / 2;
								desiredLeft = Math.max(
									arrowPadding,
									Math.min(desiredLeft, tooltipRect.width - arrowPadding - arrowWidth)
								);
								left = `${desiredLeft}px`;
							}
							if ((side === 'left' || side === 'right') && tooltipRect.height > 0) {
								const desiredCenterY = lastPointerClient.y - tooltipRect.top;
								let desiredTop = desiredCenterY - arrowHeight / 2;
								desiredTop = Math.max(
									arrowPadding,
									Math.min(desiredTop, tooltipRect.height - arrowPadding - arrowHeight)
								);
								top = `${desiredTop}px`;
							}
						}

						Object.assign(arrowElement.style, {
							left,
							top,
							right: '',
							bottom: '',
							[staticSide]: '-8px'
						});
					}
				});
			};

			updatePositionRef = updatePosition;

			floatingCleanup = autoUpdate(triggerElement, tooltipElement, updatePosition, {
				ancestorScroll: true,
				ancestorResize: true,
				elementResize: true,
				layoutShift: true,
				animationFrame: false
			});
		};

		const remove = () => {
			if (floatingCleanup) {
				floatingCleanup();
				floatingCleanup = undefined;
			}
			if (cleanupTooltipListeners) {
				cleanupTooltipListeners();
				cleanupTooltipListeners = undefined;
			}
			tooltipElement = undefined;
			isPointerOverTooltip = false;
			updatePositionRef = undefined;
			lastPointerClient = undefined;
			if (snippetMounter) {
				unmount(snippetMounter, { outro: true });
				snippetMounter = undefined;
			}
			removeTimeout = undefined;
		};

		const onFocusLost = (event?: Event) => {
			if (interactable && tooltipElement) {
				const related = (event as FocusEvent | PointerEvent | undefined)
					?.relatedTarget as Node | null;
				if (related && tooltipElement.contains(related)) {
					return;
				}
			}
			scheduleRemove();
		};
		const onTriggerPointerEnter = () => {
			isPointerOverTrigger = true;
			onFocus();
		};
		const onTriggerPointerLeave = (e: Event) => {
			isPointerOverTrigger = false;
			onFocusLost(e);
		};
		const onTriggerPointerMove = (e: PointerEvent) => {
			if (e.pointerType && e.pointerType !== 'mouse') {
				return;
			}
			lastPointerClient = { x: e.clientX, y: e.clientY };
			if (arrowFollowCursor) {
				if (updatePositionRef) {
					updatePositionRef();
				}
			}
		};

		triggerElement.addEventListener('focusin', onFocus);
		triggerElement.addEventListener('focusout', onFocusLost);
		triggerEl.addEventListener('pointerenter', onTriggerPointerEnter);
		triggerEl.addEventListener('pointerleave', onTriggerPointerLeave);
		triggerEl.addEventListener('pointermove', onTriggerPointerMove);

		return () => {
			remove?.();
			triggerElement.removeEventListener('focusin', onFocus);
			triggerElement.removeEventListener('focusout', onFocusLost);
			triggerEl.removeEventListener('pointerenter', onTriggerPointerEnter);
			triggerEl.removeEventListener('pointerleave', onTriggerPointerLeave);
			triggerEl.removeEventListener('pointermove', onTriggerPointerMove);
		};
	};
};
</script>

<script lang="ts">
import { inline } from '@floating-ui/dom';
import { fade } from 'svelte/transition';

let {
	children,
	tooltipTriggerId,
	snippetInDefaultBackground,
	...rest
}: {
	children?: Snippet<[Record<string, unknown>]>;
	tooltipTriggerId: string;
	snippetInDefaultBackground?: boolean;
} = $props();
</script>

<!-- snippetmounter -->
<div data-snippet-mounter data-tooltip-trigger-id={tooltipTriggerId} style="display: contents;">
	{#if snippetInDefaultBackground}
		{@render defaultTooltip({
			content: '',
			snippet: children,
			...rest
		})}
	{:else if children}
		{@render children?.({ ...rest })}
	{/if}
</div>

{#snippet defaultTooltip({
	content = '',
	arrow = false,
	background = true,
	inDelay = 0,
	transitionDuration = 196,
	snippet
}: OptionalTooltipOptions &
	DefaultTooltipOptions & {
		snippet?: Snippet<[Record<string, unknown>]>;
	})}
	<div role="tooltip" transition:fade|global={{ duration: transitionDuration, delay: inDelay }}>
		{#if background || snippet}
			<div
				class={(background ? 'bg' : '') + (typeof background === 'boolean' ? ' def' : '')}
				style={typeof background === 'string' ? `--bg: ${background}` : ''}>
				{#if arrow}
					<div data-tooltip-arrow></div>
				{/if}
			</div>
		{:else if arrow}
			<div data-tooltip-arrow></div>
		{/if}
		{#if snippet}
			{@render snippet({ content, arrow, background, inDelay, transitionDuration })}
		{:else}
			<p>{content}</p>
		{/if}
	</div>
{/snippet}

<style lang="postcss">
[role='tooltip'] {
	padding: 1em;
	position: absolute;
	z-index: 9999;
	width: max-content;
	border-radius: 0.5em;
	isolation: isolate;
	backdrop-filter: blur(6px);

	& > p {
		text-align: center;
	}
	box-shadow: var(--card-shadow);
	& > .bg {
		position: absolute;
		inset: 0;
		border-radius: inherit;
		z-index: -1;
		isolation: isolate;

		&::after {
			content: '';
			position: absolute;
			inset: 0;
			background: inherit;
			z-index: -2;
			background: var(--bg, var(--card-glass));
			border-radius: inherit;
		}
		& > [data-tooltip-arrow] {
			background: var(--bg, var(--card-glass));
		}
		&.def {
			&::after {
				background: var(--card-glass);
				background-color: var(--bg, color(from var(--card-color) srgb r g b / 0.5));
			}

			& > [data-tooltip-arrow] {
				background: var(--card-glass);
				background-color: var(--bg, color(from var(--card-color) srgb r g b / 0.5));
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
					z-index: -1;
				}
			}

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
				opacity: 1;
				z-index: -1;
			}
		}
	}
}

[data-tooltip-arrow] {
	position: absolute;
	width: 1.5em;
	height: 1.5em;
	transform: rotate(45deg);
	z-index: -3;
	box-shadow: var(--card-shadow);
}
</style>
