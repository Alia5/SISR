import type { Action } from 'svelte/action';
export const setStylePropertyCallback: Action<HTMLElement, undefined, {
    onsetstyleproperty: (e: CustomEvent<{ name: string; value: string; priority?: string }>) => void;
}> = (node: HTMLElement) => {
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const original = node.style.setProperty;

    const myFunc = (name: string, value: string, priority?: string) => {
        original.call(node.style, name, value, priority);
        node.dispatchEvent(new CustomEvent('setstyleproperty', { detail: { name, value, priority } }));
    };

    node.style.setProperty = myFunc;


    return {
        update() {
            // IGNORE
        },
        destroy() {
            node.style.setProperty = original;
        }
    };
};

