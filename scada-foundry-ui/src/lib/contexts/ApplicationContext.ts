import { browser, dev } from '$app/environment';
import { base, resolve } from '$app/paths';
import { page } from '$app/state';
import { ScadaForgeRequestService } from '$lib/services/ScadaForgeRequestService';
import { ScadaForgeStreamService } from '$lib/services/ScadaForgeStreamService';
import { createContext } from 'svelte';

export class ApplicationContext {
    private streamService: ScadaForgeStreamService;
    private reqeustService: ScadaForgeRequestService;

    private developmentMode: boolean;

    constructor() {
        this.developmentMode = !!dev;

        let rsUrl = resolve("/") + 'app/api';
        let wsUrl = URL.parse(resolve("/") + 'app/ws', page.url.href)?.href.replace(/^http/, 'ws');

        if (this.developmentMode && import.meta.env.VITE_BASE_URL_OVERRIDE) {
            rsUrl = import.meta.env.VITE_BASE_URL_OVERRIDE + 'app/api';
            wsUrl = import.meta.env.VITE_BASE_URL_OVERRIDE + 'app/ws';
        }
        this.reqeustService = new ScadaForgeRequestService(rsUrl);
        this.streamService = new ScadaForgeStreamService(wsUrl);
    }

    public getScadaForgeStreamService(): ScadaForgeStreamService {
        return this.streamService;
    }

    public getScadaForgeRequestService(): ScadaForgeRequestService {
        return this.reqeustService;
    }
}

export const [getApplicationContext, setApplicationContext] = createContext<ApplicationContext>();
