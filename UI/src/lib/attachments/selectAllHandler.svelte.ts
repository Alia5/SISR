// Handles Ctrl/Cmd+A and only selects the text of the element the cursor is in, not the whole page.
// Important: the listener must NOT be passive, otherwise `preventDefault()` is ignored.
export const selectAllHandler = (focusStyle?: string) => (element: HTMLElement) => {
    if (!element.hasAttribute('tabindex')) {
        element.tabIndex = 0;
    }

    let focusStyles: Record<string, string> = {};
    if (focusStyle) {
        focusStyles = focusStyle.split(';').reduce((acc, s) => {
            const [key, value] = s.split(':')
                .map((p) => p.trim());
            if (key && value) {
                acc[key] = value;
            }
            return acc;
        }, {} as Record<string, string>);
    }

    const prev: Record<string, string> = {};

    const onFocus = () => {
        Object.entries(focusStyles).forEach(([prop, val]) => {
            if (!(prop in prev)) {
                prev[prop] = element.style.getPropertyValue(prop);
            }
            element.style.setProperty(prop, val);
        });
    };

    const onBlur = () => {
        Object.keys(focusStyles).forEach((prop) => {
            if (prev[prop]) {
                element.style.setProperty(prop, prev[prop]);
            } else {
                element.style.removeProperty(prop);
            }
            delete prev[prop];
        });
    };


    const onPointerDown = () => {
        element.focus();
    };

    const onKeyDown = (e: KeyboardEvent) => {
        const isSelectAll = (e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a';
        if (!isSelectAll) {
            return;
        }

        const selection = window.getSelection();
        if (!selection) {
            return;
        }

        const range = document.createRange();
        range.selectNodeContents(element);
        selection.removeAllRanges();
        selection.addRange(range);
        e.preventDefault();
    };

    element.addEventListener('pointerdown', onPointerDown);
    element.addEventListener('focus', onFocus);
    element.addEventListener('blur', onBlur);
    element.addEventListener('keydown', onKeyDown);

    return () => {
        element.removeEventListener('pointerdown', onPointerDown);
        element.removeEventListener('focus', onFocus);
        element.removeEventListener('blur', onBlur);
        element.removeEventListener('keydown', onKeyDown);
    };
};

