<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PenOutline, PlusOutline, TrashBinOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy, onMount } from 'svelte';
	import NewCallingAssociation from './NewCallingAssociation.svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { IccpAssociation, IccpAssociationReport } from '$lib/services/ScadaForgeRequestService';
	import NewCalledAssociation from './NewCalledAssociation.svelte';

	let newTaseCalled = $state(false);
	let newTaseCalling = $state(false);
	let applicationContext = getApplicationContext();

	let { loading, serverAssociations }: { loading: boolean; serverAssociations: Array<IccpAssociationReport> } = $props();
</script>

<Table>
	<TableHead>
		<TableHeadCell></TableHeadCell>
		<TableHeadCell>Name</TableHeadCell>
		<TableHeadCell>Version</TableHeadCell>
		<TableHeadCell>Listener</TableHeadCell>
		<TableHeadCell>Side</TableHeadCell>
		<TableHeadCell>Domain</TableHeadCell>
		<TableHeadCell>Table</TableHeadCell>
		<TableHeadCell>AP Title</TableHeadCell>
		<TableHeadCell>AE Qualifier</TableHeadCell>
		<TableHeadCell>PSAP</TableHeadCell>
		<TableHeadCell>SSAP</TableHeadCell>
		<TableHeadCell>TSAP</TableHeadCell>
		<TableHeadCell>Status</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each serverAssociations as associationState}
			<TableBodyRow>
				<TableBodyCell class="w-1" rowspan={2}>
					{#if associationState.status != 'Deleted'}
						<PenOutline
							class="inline cursor-pointer pr-0.5"
							onclick={() => {
								// editAcseListenerValue = { ...acseListener };
								// editAcseListener = true;
							}}
						/><TrashBinOutline
							class="inline cursor-pointer pl-0.5"
							onclick={() => {
								// editAcseListenerValue = { ...acseListener };
								// deleteAcseListener = true;
							}}
						/>
					{/if}
				</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.name}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.version}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.source.called?.listener?.name}</TableBodyCell>
				<TableBodyCell>Local</TableBodyCell>
				<TableBodyCell>{associationState.association.localDomain}</TableBodyCell>
				<TableBodyCell>{associationState.association.localBilateralTable}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.apTitle}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.aeQualifier}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.presentationSelector}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.sessionSelector}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.tsapId}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.status}</TableBodyCell>
			</TableBodyRow>
			<TableBodyRow>
				<TableBodyCell>Remote</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDomain}</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteBilateralTable}</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDataCenterParameters.aeTitle.apTitle}</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDataCenterParameters.aeTitle.aeQualifier}</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDataCenterParameters.presentationSelector}</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDataCenterParameters.sessionSelector}</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDataCenterParameters.tsapId}</TableBodyCell>
			</TableBodyRow>
		{/each}
		{#if loading}
			<TableBodyRow>
				<TableBodyCell class="text-center" colspan={12}><Spinner class="m-auto" type="bars" color="blue" /></TableBodyCell>
			</TableBodyRow>
		{/if}
		{#if !loading && serverAssociations.length == 0}
			<TableBodyRow>
				<TableBodyCell class="text-center" colspan={12}>Empty</TableBodyCell>
			</TableBodyRow>
		{/if}
	</TableBody>
</Table>
