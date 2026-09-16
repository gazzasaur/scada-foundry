<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { AcseListenerReport, IccpAssociation, IccpAssociationState } from '$lib/services/ScadaForgeRequestService';
	import EditAcseListener from './EditAcseListener.svelte';

	let newAcseListener = $state(false);
	let applicationContext = getApplicationContext();

	let loading = $state(true);
	let acseListeners = $state<Array<AcseListenerReport>>([]);

	let listenerId = '';
	let itemsBacklog = [];

	function updateAcseListenerReport(report: AcseListenerReport) {
		let existingReport = acseListeners.find((x) => (x.id = report.id));
		if (!!existingReport) {
			Object.assign(existingReport, report);
		} else {
			acseListeners.push(report);
		}
	}

	// TODO This may cause out of order updates. Need to process websocket events after fetch.
	onMount(async () => {
		listenerId = applicationContext.getScadaForgeStreamService().addListener((message) => {
			if (message.kind != 'AcseListenerReportMessage') {
				return;
			}
			updateAcseListenerReport(message.data);
		});

		try {
			for (let report of await applicationContext.getScadaForgeRequestService().fetchAcseListeners()) {
				updateAcseListenerReport(report);
			}
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
	<div class="mb-2 flex justify-between">
		<Heading tag="h3">ACSE Listeners</Heading>
		<Button color="blue" onclick={() => (newAcseListener = true)}>Create</Button>
	</div>
	<P class="mb-2">ACSE is used by MMS to establish associations between endpoints. Listeners need to be pre-created as the underlying TCP port can be shared between associations and even between different protocols.</P>

	<AcseListenerTable {loading} {acseListeners}></AcseListenerTable>
</div>

<EditAcseListener bind:open={newAcseListener}></EditAcseListener>