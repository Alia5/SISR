/**
 * Debounces changes to an input value and exposes the settled value.
 *
 * bind to `input`,
 * `value` is updated debounced,
 * You can optionally `lock()` during async work to avoid mid-request updates.
 */
export class Debounced<T = unknown> {
    // No state! Do not re-run effect on change
    private timeout: NodeJS.Timeout | undefined;
    private updateImmediate = $state(false);
    public locked = $state(false);
    public delay = $state(0);
    private eagerUpdate = $state(false);


    public input = $state<T>();
    public value = $state<T>();


    /**
     * @param delayMs - Debounce delay in milliseconds
     * @param initial - Initial value
     */
    public constructor(public delayMs: number, initial: T = undefined as unknown as T) {
        this.input = initial;
        this.value = initial;
        this.delay = delayMs;

        $effect(() => {
            const copy = this.input;
            if (this.timeout) {
                clearTimeout(this.timeout);
            } else if (this.eagerUpdate) {
                if (!this.locked) {
                    this.value = copy;
                }
            }
            this.timeout = setTimeout(() => {
                if (this.locked) {
                    return;
                }
                if (this.updateImmediate) {
                    this.updateImmediate = false;
                }
                this.value = copy;
                clearTimeout(this.timeout);
            }, this.updateImmediate ? 0 : this.delay);
        });
    }

    /**
     * Enables or disables eager updates. When enabled, the first update
     * after an idle period will be applied immediately.
     * @returns this
     */
    public eager(eager = true) {
        this.updateImmediate = eager;
        return this;
    }

    /** Clears any pending timeout without updating the value. */
    public clear(){
        if (this.timeout) {
            clearTimeout(this.timeout);
        }
    }

    /** Prevents updates from being applied while locked. */
    public lock() {
        this.locked = true;
    }

    /**
     * Re-enables updates. Optionally applies the next update immediately.
     */
    public unlock(updateImmediate = false) {
        this.updateImmediate = updateImmediate;
        this.locked = false;
    }

    /** @returns Whether updates are currently locked. */
    public isLocked() {
        return this.locked;
    }

}
