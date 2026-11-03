<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PenOutline, PlusOutline, TrashBinOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { AcseListener, AcseListenerReport, IccpAssociation, IccpAssociationReport } from '$lib/services/ScadaForgeRequestService';
	import NewAcseListener from './EditAcseListener.svelte';
	import EditAcseListener from './EditAcseListener.svelte';
	import DeleteAcseListener from './DeleteAcseListener.svelte';

	let editAcseListener = $state(false);
	let deleteAcseListener = $state(false);
	let editAcseListenerValue = $state<undefined | AcseListener>(undefined);
	let editedAcseListener = $state({ id: '', name: '', hoost: '', port: 0 });

	let applicationContext = getApplicationContext();

	let { loading, acseListeners }: { loading: boolean; acseListeners: Array<AcseListenerReport> } = $props();
</script>

<Table>
	<TableHead>
		<TableHeadCell />
		<TableHeadCell>Name</TableHeadCell>
		<TableHeadCell>Host</TableHeadCell>
		<TableHeadCell>Port</TableHeadCell>
		<TableHeadCell>Status</TableHeadCell>
		<TableHeadCell>Associations</TableHeadCell>
	</TableHead>
	<TableBody>
		{#each acseListeners as acseListener}
			<TableBodyRow>
				<TableBodyCell class="w-1">
					{#if acseListener.status != 'Deleted'}
						<PenOutline
							class="inline cursor-pointer pr-0.5"
							onclick={() => {
								editAcseListenerValue = { ...acseListener };
								editAcseListener = true;
							}}
						/><TrashBinOutline
							class="inline cursor-pointer pl-0.5"
							onclick={() => {
								editAcseListenerValue = { ...acseListener };
								deleteAcseListener = true;
							}}
						/>
					{/if}
				</TableBodyCell>
				<TableBodyCell>{acseListener.name}</TableBodyCell>
				<TableBodyCell>{acseListener.host}</TableBodyCell>
				<TableBodyCell>{acseListener.port}</TableBodyCell>
				<TableBodyCell>{acseListener.status}</TableBodyCell>
				{#if acseListener.registrations.length == 0}
					<TableBodyCell>-</TableBodyCell>
				{/if}
				{#if acseListener.registrations.length != 0}
					<TableBodyCell>{loading}</TableBodyCell>
				{/if}
			</TableBodyRow>
		{/each}
		{#if loading}
			<TableBodyRow>
				<TableBodyCell class="text-center" colspan={5}><Spinner class="m-auto" type="bars" color="blue" /></TableBodyCell>
			</TableBodyRow>
		{/if}
		{#if !loading && acseListeners.length == 0}
			<TableBodyRow>
				<TableBodyCell class="text-center" colspan={5}>Empty</TableBodyCell>
			</TableBodyRow>
		{/if}
	</TableBody>
</Table>

<EditAcseListener bind:open={editAcseListener} existing={editAcseListenerValue} onsuccess={() => {}}></EditAcseListener>
<DeleteAcseListener bind:open={deleteAcseListener} existing={editAcseListenerValue}></DeleteAcseListener>
