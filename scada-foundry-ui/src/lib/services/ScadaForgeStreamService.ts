import { v4 as uuidv4 } from 'uuid';
import type { IccpAssociation, IccpAssociationState } from './ScadaForgeRequestService';

export interface ScadaForgeStatus {
    kind: 'ScadaForgeStatus',
    state: 'Idle' | 'Connecting' | 'Connected' | 'Failed' | 'Healthy',
    message: string,
}

export interface IccpAssociationStateMessage {
    kind: 'IccpAssociationStateMessage',
    data: IccpAssociationState,
}

export type ScadaForgeStreamServiceMessage = ScadaForgeStatus | IccpAssociationStateMessage | IccpDataPointUpdate;

export class ScadaForgeStreamService {
    private socket: WebSocket | undefined;

    private failureCount: number = 0;
    private serviceConnectionStatus: ScadaForgeStatus = { 'kind': 'ScadaForgeStatus', state: 'Idle', message: 'Client is starting. It will connect shortly.' };

    private listeners: Map<string, (message: ScadaForgeStreamServiceMessage) => void>;

    constructor(private url: string | undefined) {
        this.listeners = new Map<string, (message: ScadaForgeStreamServiceMessage) => void>();
        if (url) {
            setTimeout(() => this.connectWebSocket(), 1000 + Math.ceil(1000 * Math.random()));
        } else {
            this.serviceConnectionStatus = { 'kind': 'ScadaForgeStatus', state: 'Failed', message: 'Cannot determine websocket url.' };
        }
    }

    public addListener(listener: (message: ScadaForgeStreamServiceMessage) => void): string {
        for (let i = 0; i < 10; ++i) {
            let key = uuidv4();
            if (this.listeners.has(key)) {
                continue;
            }
            this.listeners.set(key, listener);
            this.announce(this.serviceConnectionStatus);
            return key;
        }
        throw new Error("Failed to register listener.");
    }

    public removeListener(listener: string): void {
        this.listeners.delete(listener);
    }

    private connectWebSocket() {
        if (!this.url) {
            this.serviceConnectionStatus = { 'kind': 'ScadaForgeStatus', state: 'Failed', message: 'Cannot determine websocket url.' };
            this.announce(this.serviceConnectionStatus);
            return;
        }

        this.serviceConnectionStatus = { 'kind': 'ScadaForgeStatus', state: 'Connecting', message: 'Attempting to connect to server.' };
        this.announce(this.serviceConnectionStatus);

        this.socket = new WebSocket(this.url);
        this.socket.addEventListener('error', (event) => {
            this.failureCount += 1;
        });
        this.socket.addEventListener('close', (event) => {
            if (this.failureCount > 0) {
                this.serviceConnectionStatus = { 'kind': 'ScadaForgeStatus', state: 'Failed', message: 'Client will attempt to reconnect.' };
            } else {
                this.serviceConnectionStatus = { 'kind': 'ScadaForgeStatus', state: 'Idle', message: 'Client will reconnect.' };
            }
            this.announce(this.serviceConnectionStatus);

            let backOffJitter = Math.ceil(Math.random() * (this.failureCount > 10000 ? 30000 : 3000));
            let backOff = (this.failureCount > 10000 ? 60000 : 3000 * this.failureCount) + backOffJitter;
            setTimeout(() => this.connectWebSocket(), backOff);
        });
        this.socket.addEventListener('open', (event) => {
            this.failureCount = 0;
            this.serviceConnectionStatus = { 'kind': 'ScadaForgeStatus', state: 'Connected', message: 'Connected to server.' };
            this.announce(this.serviceConnectionStatus);
        });
        this.socket.addEventListener('message', (event) => {
            let message = undefined;
            // @ts-expect-error TS does not seem to have the context 
            let messageData = JSON.parse(event.data, (key: string, value: any, context: any) => {
                if (key === 'aeQualifier') {
                    return BigInt(context.source);
                }
                return value;
            });
            if (!Object.keys(messageData).includes('kind')) {
                return;
            }
            this.announce(messageData);
        });
    }

    private announce(message: ScadaForgeStreamServiceMessage) {
        for (let [_, listener] of this.listeners) {
            try {
                listener(message);
            } catch (e) {
                console.log(`Exception caught during websocket processing: {e}`);
            }
        }
    }
}
