import { dev } from '$app/environment';
import { ScadaForgeStreamService } from '$lib/services/ScadaForgeStreamService';

export class ApplicationContext {
    private streamService: ScadaForgeStreamService;

    private baseUri: string;
    private developmentMode: boolean;

    constructor() {
        this.developmentMode = !!dev;

        this.baseUri = document.baseURI;
        if (this.developmentMode && import.meta.env.VITE_WEBSOCKET_URI_OVERRIDE) {
            this.baseUri = import.meta.env.VITE_WEBSOCKET_URI_OVERRIDE
        }

        this.streamService = new ScadaForgeStreamService(`${this.baseUri}/api/ws`);
    }

    public getScadaForgeStreamService(): ScadaForgeStreamService {
        return this.streamService;
    }
}