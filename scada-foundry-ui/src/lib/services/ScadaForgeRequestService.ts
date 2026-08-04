export type AssociationType = 'Client' | 'Server' | 'ClientBoth' | 'ServerBoth'

export interface IccpAeTitle {
    apTitle: string,
    aeQualifier: number,
}

export interface IccpDataCenterParameters {
    aeTitle: IccpAeTitle,
    tsap: string,
    ssap: string,
    psap: string,
}

export interface IccpAssociation {
    id: string,
    name: string,
    associationType: 'clientBidirectional',
    domain: string,
    bilateralTable: string,
    host: string,
    port: number,
    localDataCenterParameters: IccpDataCenterParameters,
    remoteDataCenterParameters: IccpDataCenterParameters,
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

    public async fetchIccpAssociations(): Promise<Array<IccpAssociation>> {
        return await (await fetch(`${this.url}/fetchiccpassociations`)).json() as Array<IccpAssociation>;
    }

    public async createIccpAssociation(name: string, dataCenter: String, associationType: AssociationType, host: string, port: number, localDataCenterParameters: DataCenterParameters, remoteDataCenterParameters: DataCenterParameters) {
        fetch(`${this.url}/createiccpassociation`, {
            method: 'POST', headers: { 'Content-type': 'application/json' }, body: JSON.stringify({
                name, associationType, host, port, dataCenter, localDataCenterParameters, remoteDataCenterParameters
            })
        });
    }
}
