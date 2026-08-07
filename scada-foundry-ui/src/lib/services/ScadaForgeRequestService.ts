export type AssociationType = 'clientUnidirectional' | 'clientBidirectional' | 'serverUnidirectional' | 'serverBidirectional'

export interface IccpAeTitle {
    apTitle: string,
    aeQualifier: string,
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
    associationType: 'clientUnidirectional' | 'clientBidirectional',
    domain: string,
    bilateralTable: string,
    host: string,
    port: number,
    localDataCenterParameters: IccpDataCenterParameters,
    remoteDataCenterParameters: IccpDataCenterParameters,
}

export interface AeTitle {
    apTitle: string,
    aeQualifier: string,
}

export interface DataCenterParameters {
    aeTitle: AeTitle,
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

    public async createIccpAssociation(id: string, name: string, dataCenter: String, associationType: AssociationType, host: string, port: number, localDataCenterParameters: DataCenterParameters, remoteDataCenterParameters: DataCenterParameters) {
        fetch(`${this.url}/createiccpassociation`, {
            method: 'POST', headers: { 'Content-type': 'application/json' }, body: JSON.stringify({
                id, name, associationType, host, port, dataCenter, localDataCenterParameters, remoteDataCenterParameters
            })
        });
    }
}
