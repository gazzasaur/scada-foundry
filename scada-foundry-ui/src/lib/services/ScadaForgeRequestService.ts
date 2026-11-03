import type { IccpAssociationStateMessage } from './ScadaForgeStreamService';

export type AssociationType = 'clientUnidirectional' | 'clientBidirectional' | 'serverUnidirectional' | 'serverBidirectional';

export interface AcseAeTitle {
	apTitle: string;
	aeQualifier: BigInt;
}

export interface AcseIdentityParameters {
	tsapId?: String;
	sessionSelector?: String;
	presentationSelector?: String;
	aeTitle: AcseAeTitle;
}

export interface IccpAssociationListener {
	id: string;
	name: string;
}

export interface IccpAssociationSource {
	called?: {
		listener?: IccpAssociationListener;
	};
	calling?: {
		host: string;
		port: number;
	};
}

export interface IccpAssociation {
	id: string;
	name: string;
	version: string;
	enabled: boolean;

	localDomain: string;
	remoteDomain: string;
	localBilateralTable: string;
	remoteBilateralTable: string;

	source: IccpAssociationSource;

	localDataCenterParameters: AcseIdentityParameters;
	remoteDataCenterParameters: AcseIdentityParameters;
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

export interface IccpAssociationReport {
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

	public async createAcseListener(listener: AcseListener): Promise<AcseListenerReport> {
		return (await this.postWithResult(`${this.url}/createacselistener`, listener)) as AcseListenerReport;
	}

	public async updateAcseListener(listener: AcseListener): Promise<AcseListenerReport> {
		return (await this.postWithResult(`${this.url}/updateacselistener`, listener)) as AcseListenerReport;
	}

	public async deleteAcseListener(id: string): Promise<void> {
		return await this.postWithoutResult(`${this.url}/deleteacselistener`, { id: id });
	}

	public async fetchIccpAssociations(): Promise<Array<IccpAssociationReport>> {
		return this.fromJson(await (await fetch(`${this.url}/fetchiccpassociations`)).text()) as Array<IccpAssociationReport>;
	}

	public async fetchIccpDatapoints(): Promise<Array<IccpAssociation>> {
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

	private async postWithoutResult(url: string, data: any): Promise<void> {
		await fetch(url, {
			method: 'POST',
			headers: { 'Content-type': 'application/json' },
			body: this.toJson(data)
		});
	}

	private async postWithResult(url: string, data: any): Promise<any> {
		return this.fromJson(
			await (
				await fetch(url, {
					method: 'POST',
					headers: { 'Content-type': 'application/json' },
					body: this.toJson(data)
				})
			).text()
		);
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
