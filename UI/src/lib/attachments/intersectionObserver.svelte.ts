

export const intersectionObserver = (
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    callback = (isIntersecting: boolean) => { },
    threshold = 0.5,
    call_w_state = false
) => (element: HTMLElement) => {
    const observer = new IntersectionObserver(
        (entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    callback(true);
                }
                if (call_w_state) {
                    callback(entry.isIntersecting);
                }
            });
        },
        {
            threshold: threshold
        }
    );
    observer.observe(element);
    return () => {
        observer.disconnect();
    };
};

