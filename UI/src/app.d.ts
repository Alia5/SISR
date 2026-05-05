import '@poppanator/sveltekit-svg/dist/svg.d.ts';
import 'unplugin-fonts/client';
import 'unplugin-icons/types/svelte';


// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
    namespace App {
        interface Error {
            message: string;
            err?: unknown;
            [key: string]: unknown;
        }
        // interface Locals {}
        interface PageData {
            theme?: 'light' | 'dark';
        }
        // interface PageState {}
        // interface Platform {}
    }

    interface Window {
        invalidateAll: typeof import('$app/navigation').invalidateAll;
    }
}


export { };

