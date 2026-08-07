<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onMount } from 'svelte';
	import NewCallingAssociation from './NewCallingAssociation.svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { IccpAssociation } from '$lib/services/ScadaForgeRequestService';
	import NewCalledAssociation from './NewCalledAssociation.svelte';

	let newTaseCalled = $state(false);
	let newTaseCalling = $state(false);
	let applicationContext = getApplicationContext();

	let loading = $state(true);
	let associations = $state<Array<IccpAssociation>>([]);

	onMount(async () => {
		try {
			associations = await applicationContext.getScadaForgeRequestService().fetchIccpAssociations();
			loading = false;
		} catch (e) {
			console.log(e);
		}
	});
</script>

<div class="p-10">
	<Heading tag="h3" class="mb-3">TASE.2 Associations</Heading>
	<P class="mb-2">ICCP Associations connect data centers to each other.</P>
	<P>The calling association will initiate a connection and act as an ICCP client but may also act as a server subject to negotiation.</P>
	<P class="mb-5">The called association will wait for a connection and act as an ICCP server but may also act as a client subject to negotiation.</P>
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
					{#each associations as association}
						{#if ['clientUnidirectional', 'clientBidirectional'].includes(association.associationType)}
							<TableBodyRow>
								<TableBodyCell>{association.name}</TableBodyCell>
								<TableBodyCell>{association.associationType}</TableBodyCell>
								<TableBodyCell>{association.bilateralTable}</TableBodyCell>
								<TableBodyCell>{association.host}</TableBodyCell>
								<TableBodyCell>{association.port}</TableBodyCell>
								<TableBodyCell
									><div class="border-b-2">{association.localDataCenterParameters.aeTitle.apTitle}</div>
									<div>{association.remoteDataCenterParameters.aeTitle.apTitle}</div></TableBodyCell
								>
								<TableBodyCell
									><div class="border-b-2">{association.localDataCenterParameters.aeTitle.aeQualifier}</div>
									<div>{association.remoteDataCenterParameters.aeTitle.aeQualifier}</div></TableBodyCell
								>
								<TableBodyCell
									><div class="border-b-2">{association.localDataCenterParameters.tsap}</div>
									<div>{association.remoteDataCenterParameters.tsap}</div></TableBodyCell
								>
								<TableBodyCell
									><div class="border-b-2">{association.localDataCenterParameters.ssap}</div>
									<div>{association.remoteDataCenterParameters.ssap}</div></TableBodyCell
								>
								<TableBodyCell
									><div class="border-b-2">{association.localDataCenterParameters.psap}</div>
									<div>{association.remoteDataCenterParameters.psap}</div></TableBodyCell
								>
							</TableBodyRow>
						{/if}
					{/each}
				</TableBody>
			</Table>
			{#if loading}
				<div class="pt-2 text-center">
					<Spinner type="bars" color="blue" />
				</div>
			{/if}
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
					<TableHeadCell>Host</TableHeadCell>
					<TableHeadCell>Port</TableHeadCell>
					<TableHeadCell>TSAP</TableHeadCell>
					<TableHeadCell>SSAP</TableHeadCell>
					<TableHeadCell>PSAP</TableHeadCell>
					<TableHeadCell>AP Title</TableHeadCell>
					<TableHeadCell>AE Qualifier</TableHeadCell>
					<TableHeadCell>Status</TableHeadCell>
					<TableHeadCell>Controls</TableHeadCell>
				</TableHead>
				<TableBody></TableBody>
			</Table>
			<div class="pt-2 text-center">
				<Spinner type="bars" color="blue" />
				<!-- <P class="inline">Empty</P> -->
			</div>
		</TabItem>
	</Tabs>
</div>

<NewCallingAssociation bind:open={newTaseCalling} />
<NewCalledAssociation bind:open={newTaseCalled} />
