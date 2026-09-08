<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy, onMount } from 'svelte';
	import NewCallingAssociation from './NewCallingAssociation.svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { IccpAssociation, IccpAssociationState } from '$lib/services/ScadaForgeRequestService';
	import NewCalledAssociation from './NewCalledAssociation.svelte';

	let newTaseCalled = $state(false);
	let newTaseCalling = $state(false);
	let applicationContext = getApplicationContext();

	let { loading, clientAssociations } = $props();
</script>

<Table>
	<TableHead>
		<TableHeadCell>Name</TableHeadCell>
		<TableHeadCell>Type</TableHeadCell>
		<TableHeadCell>Domain</TableHeadCell>
		<TableHeadCell>Table</TableHeadCell>
		<TableHeadCell>Host</TableHeadCell>
		<TableHeadCell>Port</TableHeadCell>
		<TableHeadCell>Side</TableHeadCell>
		<TableHeadCell>AP Title</TableHeadCell>
		<TableHeadCell>AE Qualifier</TableHeadCell>
		<TableHeadCell>TSAP</TableHeadCell>
		<TableHeadCell>SSAP</TableHeadCell>
		<TableHeadCell>PSAP</TableHeadCell>
		<TableHeadCell>Status</TableHeadCell>
		<TableHeadCell>Controls</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each clientAssociations as associationState}
			<TableBodyRow>
				<TableBodyCell rowspan={2}>{associationState.association.name}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.associationType}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.domain}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.bilateralTable}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.host}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.association.port}</TableBodyCell>
				<TableBodyCell>Local</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.apTitle}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.aeQualifier}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.tsap}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.ssap}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.psap}</TableBodyCell>
				<TableBodyCell rowspan={2}>{associationState.status}</TableBodyCell>
				<TableBodyCell rowspan={2}></TableBodyCell>
			</TableBodyRow>
			<TableBodyRow>
				<TableBodyCell>Remote</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDataCenterParameters.aeTitle.apTitle}</TableBodyCell>
				<TableBodyCell>{associationState.association.remoteDataCenterParameters.aeTitle.aeQualifier}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.tsap}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.ssap}</TableBodyCell>
				<TableBodyCell>{associationState.association.localDataCenterParameters.psap}</TableBodyCell>
			</TableBodyRow>
		{/each}
		{#if loading}
			<TableBodyRow>
				<TableBodyCell class="text-center" colspan={12}><Spinner class="m-auto" type="bars" color="blue" /></TableBodyCell>
			</TableBodyRow>
		{/if}
		{#if !loading && clientAssociations.length == 0}
			<TableBodyRow>
				<TableBodyCell class="text-center" colspan={12}>Empty</TableBodyCell>
			</TableBodyRow>
		{/if}
	</TableBody>
</Table>
