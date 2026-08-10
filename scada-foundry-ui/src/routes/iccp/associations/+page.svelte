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

	let loading = $state(true);
	let associations = $state<Array<IccpAssociationState>>([]);
	let clientAssociations = $derived(associations.filter((state) => ['clientUnidirectional', 'clientBidirectional'].includes(state.association.associationType)));
	let serverAssociations = $derived(associations.filter((state) => ['serverUnidirectional', 'serverBidirectional'].includes(state.association.associationType)));

	let listenerId = '';
	onMount(async () => {
		listenerId = applicationContext.getScadaForgeStreamService().addListener((message) => {
			if (message.kind != 'IccpAssociationStateMessage') {
				return;
			}
			let association = associations.find((state) => state.association.id == message.data.association.id);
			if (!association) {
				associations.push(message.data);
			} else {
				Object.assign(association, { ...message.data });
			}
		});

		try {
			associations = await applicationContext.getScadaForgeRequestService().fetchIccpAssociations();
			loading = false;
		} catch (e) {
			console.log(e);
		}
	});

	onDestroy(async () => {
		if (!!listenerId) {
			applicationContext.getScadaForgeStreamService().removeListener(listenerId);
		}
	});
</script>

<div class="p-10">
	<Heading tag="h3" class="mb-3">TASE.2 Associations</Heading>
	<P class="mb-2">ICCP Associations connect data centers to each other.</P>
	<P>The calling association will initiate a connection and act as an ICCP client but may also act as a server subject to negotiation.</P>
	<P>The called association will wait for a connection and act as an ICCP server but may also act as a client subject to negotiation.</P>
	<P class="mb-5">Properties on the top of each row are local and properties on the bottom of each row are remote.</P>
	<Tabs classes={{ active: 'p-4 text-white bg-blue-500 rounded-t-lg dark:bg-blue-600 dark:text-white' }}>
		<TabItem open title="Calling">
			<div class="mb-2 flex justify-between">
				<Heading tag="h3">Calling Associations</Heading>
				<Button color="blue" onclick={() => (newTaseCalling = true)}>Create</Button>
			</div>
			<Table>
				<TableHead>
					<TableHeadCell>Name</TableHeadCell>
					<TableHeadCell>Type</TableHeadCell>
					<TableHeadCell>Domain</TableHeadCell>
					<TableHeadCell>Table</TableHeadCell>
					<TableHeadCell>Host</TableHeadCell>
					<TableHeadCell>Port</TableHeadCell>
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
							<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.apTitle}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.aeQualifier}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.tsap}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.ssap}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.psap}</TableBodyCell>
							<TableBodyCell rowspan={2}>{associationState.status}</TableBodyCell>
							<TableBodyCell rowspan={2}></TableBodyCell>
						</TableBodyRow>
						<TableBodyRow>
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
		</TabItem>
		<TabItem title="Called">
			<div class="mb-2 flex justify-between">
				<Heading tag="h3">Called Associations</Heading>
				<Button color="blue" onclick={() => (newTaseCalled = true)}>Create</Button>
			</div>
			<Table>
				<TableHead>
					<TableHeadCell>Name</TableHeadCell>
					<TableHeadCell>Type</TableHeadCell>
					<TableHeadCell>Domain</TableHeadCell>
					<TableHeadCell>Table</TableHeadCell>
					<TableHeadCell>Host</TableHeadCell>
					<TableHeadCell>Port</TableHeadCell>
					<TableHeadCell>AP Title</TableHeadCell>
					<TableHeadCell>AE Qualifier</TableHeadCell>
					<TableHeadCell>TSAP</TableHeadCell>
					<TableHeadCell>SSAP</TableHeadCell>
					<TableHeadCell>PSAP</TableHeadCell>
					<TableHeadCell>Status</TableHeadCell>
					<TableHeadCell>Controls</TableHeadCell>
				</TableHead>
				<TableBody>
					{#each serverAssociations as associationState}
						<TableBodyRow>
							<TableBodyCell rowspan={2}>{associationState.association.name}</TableBodyCell>
							<TableBodyCell rowspan={2}>{associationState.association.associationType}</TableBodyCell>
							<TableBodyCell rowspan={2}>{associationState.association.domain}</TableBodyCell>
							<TableBodyCell rowspan={2}>{associationState.association.bilateralTable}</TableBodyCell>
							<TableBodyCell rowspan={2}>{associationState.association.host}</TableBodyCell>
							<TableBodyCell rowspan={2}>{associationState.association.port}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.apTitle}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.aeTitle.aeQualifier}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.tsap}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.ssap}</TableBodyCell>
							<TableBodyCell>{associationState.association.localDataCenterParameters.psap}</TableBodyCell>
							<TableBodyCell rowspan={2}>{associationState.status}</TableBodyCell>
							<TableBodyCell rowspan={2}></TableBodyCell>
						</TableBodyRow>
						<TableBodyRow>
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
					{#if !loading && serverAssociations.length == 0}
						<TableBodyRow>
							<TableBodyCell class="text-center" colspan={12}>Empty</TableBodyCell>
						</TableBodyRow>
					{/if}
				</TableBody>
			</Table>
		</TabItem>
	</Tabs>
</div>

<NewCallingAssociation bind:open={newTaseCalling} />
<NewCalledAssociation bind:open={newTaseCalled} />
