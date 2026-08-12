<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { IccpAssociation, IccpAssociationState, IccpDataPoint } from '$lib/services/ScadaForgeRequestService';
	import NewClientDataPoint from './NewClientDataPoint.svelte';

	let newTaseCalled = $state(false);
	let newClientDataPoint = $state(false);
	let applicationContext = getApplicationContext();

	let loading = $state(true);
	let dataPoints = $state<Array<IccpDataPoint>>([]);
	let associations = $state<Array<IccpAssociationState>>([]);

	let clientAssociations = $derived(associations.filter((state) => ['clientUnidirectional', 'clientBidirectional'].includes(state.association.associationType)));
	let serverAssociations = $derived(associations.filter((state) => ['serverUnidirectional', 'serverBidirectional'].includes(state.association.associationType)));
	let clientDataPoints = $derived(dataPoints.filter((dataPoint) => clientAssociations.map((state) => state.association.id).includes(dataPoint.associationId)));
	let serverDataPoints = $derived(dataPoints.filter((dataPoint) => serverAssociations.map((state) => state.association.id).includes(dataPoint.associationId)));

	let listenerId = '';
	onMount(async () => {
		listenerId = applicationContext.getScadaForgeStreamService().addListener((message) => {
			if (message.kind != 'IccpAssociationStateMessage') {
				return;
			}
			// let association = data_points.find((state) => state.association.id == message.data.association.id);
			// if (!association) {
			// 	data_points.push(message.data);
			// } else {
			// 	Object.assign(association, { ...message.data });
			// }
		});

		try {
			dataPoints = await applicationContext.getScadaForgeRequestService().fetchIccpDataPoints();
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
	<Heading tag="h3" class="mb-3">TASE.2 Data Points</Heading>
	<P class="mb-5">Client data points are received from remote data centers. Server data points are sent from this data center.</P>
	<Tabs classes={{ active: 'p-4 text-white bg-blue-500 rounded-t-lg dark:bg-blue-600 dark:text-white' }}>
		<TabItem open title="Client">
			<div class="mb-2 flex justify-between">
				<Heading tag="h3">Client Data Points</Heading>
				<Button color="blue" onclick={() => (newClientDataPoint = true)}>Create</Button>
			</div>
			<Table>
				<TableHead>
					<TableHeadCell>Name</TableHeadCell>
				</TableHead>
				<TableBody>
					{#each dataPoints as dataPoint}
						<TableBodyRow>
							<TableBodyCell></TableBodyCell>
						</TableBodyRow>
					{/each}
					{#if loading}
						<TableBodyRow>
							<TableBodyCell class="text-center" colspan={12}><Spinner class="m-auto" type="bars" color="blue" /></TableBodyCell>
						</TableBodyRow>
					{/if}
					{#if !loading && dataPoints.length == 0}
						<TableBodyRow>
							<TableBodyCell class="text-center" colspan={12}>Empty</TableBodyCell>
						</TableBodyRow>
					{/if}
				</TableBody>
			</Table>
		</TabItem>
		<TabItem title="Server">
			<div class="mb-2 flex justify-between">
				<Heading tag="h3">Server Data Points</Heading>
				<Button color="blue" onclick={() => (newTaseCalled = true)}>Create</Button>
			</div>
			<Table>
				<TableHead>
					<TableHeadCell>Name</TableHeadCell>
				</TableHead>
				<TableBody>
					{#each dataPoints as dataPoint}
						<TableBodyRow>
							<TableBodyCell></TableBodyCell>
						</TableBodyRow>
					{/each}
					{#if loading}
						<TableBodyRow>
							<TableBodyCell class="text-center" colspan={12}><Spinner class="m-auto" type="bars" color="blue" /></TableBodyCell>
						</TableBodyRow>
					{/if}
					{#if !loading && dataPoints.length == 0}
						<TableBodyRow>
							<TableBodyCell class="text-center" colspan={12}>Empty</TableBodyCell>
						</TableBodyRow>
					{/if}
				</TableBody>
			</Table>
		</TabItem>
	</Tabs>
</div>

<NewClientDataPoint bind:open={newClientDataPoint} association_states={clientAssociations} />
