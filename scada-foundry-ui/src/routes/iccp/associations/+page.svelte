<script lang="ts">
	import { getApplicationContext, type ApplicationContext } from '$lib/contexts/ApplicationContext';
	import {
		A,
		Alert,
		Button,
		Card,
		Heading,
		Hr,
		Modal,
		P,
		Table,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell
	} from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';
	import { getContext, onMount } from 'svelte';
	import NewInitiatorAssociation from './NewInitiatorAssociation.svelte';

	let newTaseInitiator = $state(false);

	let applicationcontext = getApplicationContext();
</script>

<div class="p-10">
	<div>
		<Heading tag="h3" class="mb-3"
			>TASE.2 Initiator<A onclick={() => (newTaseInitiator = true)} class="m-3"
				><PlusOutline class="mr-2 h-5 w-5 cursor-pointer" /></A
			></Heading
		>
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
			<TableBody>
				<TableBodyRow>
					<TableBodyCell>Apple MacBook Pro 17"</TableBodyCell>
					<TableBodyCell>Silver</TableBodyCell>
					<TableBodyCell>Laptop</TableBodyCell>
					<TableBodyCell>$2999</TableBodyCell>
				</TableBodyRow>
				<TableBodyRow>
					<TableBodyCell>Microsoft Surface Pro</TableBodyCell>
					<TableBodyCell>White</TableBodyCell>
					<TableBodyCell>Laptop PC</TableBodyCell>
					<TableBodyCell>$1999</TableBodyCell>
				</TableBodyRow>
				<TableBodyRow>
					<TableBodyCell>Magic Mouse 2</TableBodyCell>
					<TableBodyCell>Black</TableBodyCell>
					<TableBodyCell>Accessories</TableBodyCell>
					<TableBodyCell>$99</TableBodyCell>
				</TableBodyRow>
			</TableBody>
		</Table>

		<Heading tag="h3" class="mb-3"
			>TASE.2 Responder<A class="m-3"><PlusOutline class="mr-2 h-5 w-5 cursor-pointer" /></A
			></Heading
		>
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
			<TableBody>
				<TableBodyRow>
					<TableBodyCell>Apple MacBook Pro 17"</TableBodyCell>
					<TableBodyCell>Silver</TableBodyCell>
					<TableBodyCell>Laptop</TableBodyCell>
					<TableBodyCell>$2999</TableBodyCell>
				</TableBodyRow>
				<TableBodyRow>
					<TableBodyCell>Microsoft Surface Pro</TableBodyCell>
					<TableBodyCell>White</TableBodyCell>
					<TableBodyCell>Laptop PC</TableBodyCell>
					<TableBodyCell>$1999</TableBodyCell>
				</TableBodyRow>
				<TableBodyRow>
					<TableBodyCell>Magic Mouse 2</TableBodyCell>
					<TableBodyCell>Black</TableBodyCell>
					<TableBodyCell>Accessories</TableBodyCell>
					<TableBodyCell>$99</TableBodyCell>
				</TableBodyRow>
			</TableBody>
		</Table>
	</div>
</div>

<NewInitiatorAssociation bind:open={newTaseInitiator} />
