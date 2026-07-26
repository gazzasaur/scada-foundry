import { v4 as uuidv4 } from 'uuid';

export interface ScadaForgeStatus {
    kind: 'ScadaForgeStatus',
    connected: false,
    message: string,
}

export interface IccpDataPointUpdate {
    kind: 'IccpDataPointUpdate'
}

export class ScadaForgeStreamService {
    private socket: WebSocket;
    private failureCount: number = 0;

    private listeners: Map<string, (message: ScadaForgeStatus | IccpDataPointUpdate) => void>;

    constructor(private url: string) {
        let websocketUrl = url;

        this.socket = new WebSocket(url);
        setTimeout(() => this.connectWebSocket(), 3 + Math.ceil(3*Math.random()));
        this.listeners = new Map<string, (message: ScadaForgeStatus | IccpDataPointUpdate) => void>();
    }

    public addListener(listener: (message: ScadaForgeStatus | IccpDataPointUpdate) => void): string {
        for (let i = 0; i < 10; ++i) {
            let key = uuidv4();
            this.listeners.set(key, listener);
            return key;
        }
        throw new Error("Failed to register listener.");
    }

    public removeListener(listener: string): void {
        this.listeners.delete(listener);
    }

    private connectWebSocket() {
        this.socket = new WebSocket(this.url);
        this.socket.addEventListener('error', (event) => {
            this.failureCount += 1;
        });
        this.socket.addEventListener('close', (event) => {
            let backOffJitter = Math.ceil(Math.random()*(this.failureCount > 10 ? 30 : 3));
            let backOff = (this.failureCount > 10 ? 60 : 3 * this.failureCount) + backOffJitter;
            setTimeout(() => this.connectWebSocket(), backOff);
        });
        this.socket.addEventListener('open', (event) => {
            this.failureCount = 0;
        });
        this.socket.addEventListener('message', (event) => {
            let message = undefined;
            let messageData = event.data;
            if (Object.keys(messageData).includes('kind')) {
                switch (messageData['kind']) {
                    case 'ScadaForgeStatus':
                        message = event.data as ScadaForgeStatus;
                }
            }
            if (!message) {
                return;
            }

            for (let [_, listener] of this.listeners) {
                try {
                    return listener(message);
                } catch (e) {
                    console.log(`Exception caught during websocket processing: {e}`);
                }
            }
        });
    }
}
