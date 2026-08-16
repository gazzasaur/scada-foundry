<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import { A, Alert, Button, Card, Heading, Hr, Modal, P, Progressradial, Spinner, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { Tabs, TabItem } from 'flowbite-svelte';
	import type { IccpAssociation, IccpAssociationState, IccpDataPoint } from '$lib/services/ScadaForgeRequestService';

	let applicationContext = getApplicationContext();

	let loading = $state(true);

	let listenerId = '';
	onMount(async () => {
		listenerId = applicationContext.getScadaForgeStreamService().addListener((message) => {
			if (message.kind != 'IccpAssociationStateMessage') {
				return;
			}
		});

		try {
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
	<Heading tag="h3" class="mb-3">TASE.2 Transfer Sets</Heading>
</div>
