class ScadaForgeStreamService {
    public socket: WebSocket;

    constructor(private url: string) {
        this.socket = new WebSocket(url);
    }
}