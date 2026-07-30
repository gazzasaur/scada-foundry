<script lang="ts">
  import { A, Heading, P, Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from "flowbite-svelte";
	import { PlusOutline } from "flowbite-svelte-icons";

  let newTaseInitiator = $state(false);
</script>

<div class="p-10">
	<div>
		<Heading tag="h3" class="mb-3"
			>Virtual Control Centers<A onclick={() => (newTaseInitiator = true)} class="m-3"
				><PlusOutline class="mr-2 h-5 w-5 cursor-pointer" /></A
			></Heading
		>
    <P class="mb-5">
      A Virtual Vontrol Venter (VCC) is an ICCP entity used as a container for data points, devices, domains and associations.
      Supports multiple VCCs per device. Each VCC operates as a namespace and security boundary.
      Associations cannot directly access data points across VCC boundaries.
      The same data point names, devices and domains may be used across VCCs.
      Policies may also be specified per data point to further scope down access.
    </P>
		<Table>
			<TableHead>
				<TableHeadCell>Name</TableHeadCell>
				<TableHeadCell>Description</TableHeadCell>
			</TableHead>
			<TableBody>
				<TableBodyRow>
					<TableBodyCell>Apple MacBook Pro 17"</TableBodyCell>
					<TableBodyCell>Silver</TableBodyCell>
				</TableBodyRow>
				<TableBodyRow>
					<TableBodyCell>Microsoft Surface Pro</TableBodyCell>
					<TableBodyCell>White</TableBodyCell>
				</TableBodyRow>
				<TableBodyRow>
					<TableBodyCell>Magic Mouse 2</TableBodyCell>
					<TableBodyCell>Black</TableBodyCell>
				</TableBodyRow>
			</TableBody>
		</Table>
	</div>
</div>
