export type AssociationType = 'Client' | 'Server' | 'ClientBoth' | 'ServerBoth'

export interface IccpDataCenter {
    uuid: string,
    name: string,
}

export interface DataCenterParameters {
    apTitle: string,
    aeQualifier: string,
    tsap: string,
    ssap: string,
    psap: string,
}

export class ScadaForgeRequestService {
    constructor(private url: string) {
    }

    public async fetchDataCenters(): Promise<Array<IccpDataCenter>> {
        return await (await fetch(`${this.url}/fetchiccpdatacenters`)).json() as Array<IccpDataCenter>;
    }

    public async createIccpAssociation(name: string, dataCenter: String, associationType: AssociationType, host: string, port: number, localDataCenterParameters: DataCenterParameters, remoteDataCenterParameters: DataCenterParameters) {
        fetch(`${this.url}/createiccpassociation`, {
            method: 'POST', headers: {'Content-type': 'application/json'}, body: JSON.stringify({
                name, associationType, host, port, dataCenter, localDataCenterParameters, remoteDataCenterParameters
            })
        });
    }
}
