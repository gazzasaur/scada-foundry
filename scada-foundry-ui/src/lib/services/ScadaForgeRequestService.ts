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

export interface AcseListener {
	id: string;
	name: string;
	host: string;
	port: number;
}

export interface AcseListenerRegistration {}

export interface AcseListenerReport {
	id: string;
	name: string;
	host: string;
	port: number;
	status: string;

	registrations: Array<AcseListenerRegistration>;
}

export interface IccpAssociationState {
	association: IccpAssociation;
	status: string;
}

export interface IccpDataPoint {
	associationId: string;
	dataPointName: string;
	dataPointDomain: string; // Can be blank
	dataPointType: 'State' | 'RealQ' | 'DiscreteQ'; // Can be blank
}

export class ScadaForgeRequestService {
	constructor(private url: string) {}

	public async fetchAcseListeners(): Promise<Array<AcseListenerReport>> {
		return this.fromJson(await (await fetch(`${this.url}/fetchacselisteners`)).text()) as Array<AcseListenerReport>;
	}

	public async createAcseListener(listener: AcseListener): Promise<String> {
		return await this.postWithReturnId(`${this.url}/createacselistener`, listener);
	}

	public async updateAcseAssociation(listener: IccpAssociation): Promise<String> {
		return await this.postWithReturnId(`${this.url}/updateiccpassociation`, listener);
	}

	public async fetchIccpAssociations(): Promise<Array<IccpAssociationState>> {
		return this.fromJson(await (await fetch(`${this.url}/fetchiccpassociations`)).text()) as Array<IccpAssociationState>;
	}

	public async fetchIccpAssociations(): Promise<Array<IccpAssociation>> {
		return (await (await fetch(`${this.url}/fetchiccpdatapoints`)).json()) as Array<IccpDataPoint>;
	}

	public async createIccpAssociation(association: IccpAssociation): Promise<String> {
		return await (
			await fetch(`${this.url}/createiccpassociation`, {
				method: 'POST',
				headers: { 'Content-type': 'application/json' },
				body: this.toJson(association)
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
			body: this.toJson(dataPoint)
		});
		return;
	}

	private async postWithReturnId(url: string, data: any): Promise<string> {
		return await (
			await fetch(url, {
				method: 'POST',
				headers: { 'Content-type': 'application/json' },
				body: this.toJson(data)
			})
		).text();
	}

	private toJson(data: any): string {
		return JSON.stringify(data, (key: string, value: any) => {
			if (key === 'aeQualifier') {
				// @ts-expect-error TS does not seem to have the context
				return JSON.rawJSON(value.toString());
			}
			return value;
		});
	}

	private fromJson(data: string): any {
		// @ts-expect-error TS does not seem to have the context
		return JSON.parse(data, (key: string, value: any, context: any) => {
			if (key === 'aeQualifier') {
				return BigInt(context.source);
			}
			return value;
		});
	}
}
