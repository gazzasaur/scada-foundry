import type { IccpAssociationStateMessage } from './ScadaForgeStreamService';

export type AssociationType = 'clientUnidirectional' | 'clientBidirectional' | 'serverUnidirectional' | 'serverBidirectional';

export interface IccpAeTitle {
	apTitle: string;
	aeQualifier: BigInt;
}

export interface IccpDataCenterParameters {
	aeTitle: IccpAeTitle;
	tsap: string;
	ssap: string;
	psap: string;
}

export interface IccpAssociation {
	id: string;
	name: string;
	associationType: AssociationType;
	domain: string;
	bilateralTable: string;
	host: string;
	port: number;
	localDataCenterParameters: IccpDataCenterParameters;
	remoteDataCenterParameters: IccpDataCenterParameters;
}

export interface IccpAssociationState {
	association: IccpAssociation;
	status: string;
}

export interface IccpDataPoint {
	associationId: string;
	dataPointName: { Vcc: string } | { Icc: [string, string] };
}

export class ScadaForgeRequestService {
	constructor(private url: string) {}

	public async fetchIccpAssociations(): Promise<Array<IccpAssociationState>> {
		// @ts-expect-error TS does not seem to have the context
		return JSON.parse(await (await fetch(`${this.url}/fetchiccpassociations`)).text(), (key: string, value: any, context: any) => {
			if (key === 'aeQualifier') {
				return BigInt(context.source);
			}
			return value;
		}) as Array<IccpAssociationState>;
	}

	public async createIccpAssociation(association: IccpAssociation): Promise<String> {
		return await (
			await fetch(`${this.url}/createiccpassociation`, {
				method: 'POST',
				headers: { 'Content-type': 'application/json' },
				body: JSON.stringify(association, (key: string, value: any) => {
					if (key === 'aeQualifier') {
						// @ts-expect-error TS does not seem to have the context
						return JSON.rawJSON(value.toString());
					}
					return value;
				})
			})
		).text();
	}

	public async fetchIccpDataPoints(): Promise<Array<IccpDataPoint>> {
		return (await (await fetch(`${this.url}/fetchiccpdatapoints`)).json()) as Array<IccpDataPoint>;
	}

	public async createIccpDataPoint(dataPoint: IccpDataPoint): Promise<void> {
		await fetch(`${this.url}/createiccpdatapoint`, {
			method: 'POST',
			headers: { 'Content-type': 'application/json' },
			body: JSON.stringify(dataPoint)
		});
		return;
	}
}
