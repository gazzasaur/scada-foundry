<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PenOutline, PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { AcseListenerReport, IccpAssociation, IccpAssociationState } from '$lib/services/ScadaForgeRequestService';
	import NewAcseListener from './EditAcseListener.svelte';
	import EditAcseListener from './EditAcseListener.svelte';

	let applicationContext = getApplicationContext();

	let { loading, acseListeners }: { loading: boolean, acseListeners: Array<AcseListenerReport> } = $props();
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
				<TableBodyCell class="w-0.5"><PenOutline class="cursor-pointer" onclick /></TableBodyCell>
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

<EditAcseListener bind:open={newAcseListener}></EditAcseListener>