<script lang="ts">
	import { A, Alert, Button, Heading, Modal, P, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from 'flowbite-svelte';
	import { PlusOutline } from 'flowbite-svelte-icons';

	let defaultModal = $state(false);
</script>

<div class="p-10">
	<Heading tag="h3" class="mb-3">Initiator<A onclick={() => (defaultModal = true)} class="m-3"><PlusOutline class="w-5 h-5 mr-2 cursor-pointer" /></A></Heading>
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

	<Heading tag="h3" class="mb-3">Responder<A class="m-3"><PlusOutline class="w-5 h-5 mr-2 cursor-pointer" /></A></Heading>
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

<Modal title="Terms of Service" form bind:open={defaultModal} onaction={({ action }) => alert(`Handle "${action}"`)}>
  <P>With less than a month to go before the European Union enacts new consumer privacy laws for its citizens, companies around the world are updating their terms of service agreements to comply.</P>
  <P>
    The European Union’s General Data Protection Regulation (G.D.P.R.) goes into effect on May 25 and is meant to ensure a common set of data rights in the European Union. It requires organizations to
    notify users as soon as possible of high-risk data breaches that could personally affect them.
  </P>

  {#snippet footer()}
    <Button type="submit" value="success">I accept</Button>
    <Button type="submit" value="decline" color="alternative">Decline</Button>
  {/snippet}
</Modal>
