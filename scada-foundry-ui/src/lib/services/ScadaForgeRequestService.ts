export type AssociationType = 'Client' | 'Server' | 'ClientBoth' | 'ServerBoth'

export interface DataCenterParameters {
    apTitle: string,
    aeQualifier: string,
    tsap: string,
    ssap: string,
    psap: string,
}

export interface IccpInitiatorAssociation {
    
}

export class ScadaForgeRequestService {
    constructor(private url: string) {
    }

    public createIccpAssociation(name: string, associationType: AssociationType, host: string, port: number, localDataCenterParameters: DataCenterParameters, remoteDataCenterParameters: DataCenterParameters) {

    }
}