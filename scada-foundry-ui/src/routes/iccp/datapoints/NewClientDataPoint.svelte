<script lang="ts">
	import { getApplicationContext } from '$lib/contexts/ApplicationContext';
	import { Button, Heading, Input, Label, Modal, Select } from 'flowbite-svelte';

	const createBlankEntry = () => {
		return {
			associationId: '',

			name: '',
			domain: '',
			dataPointType: 'RealQ'
		};
	};

	let { open = $bindable(false), association_states } = $props();

	let context = getApplicationContext();
	let dataPoint = $state(createBlankEntry());
</script>

<Modal title="New TASE.2 Data Point" form class="border" {open} outsideclose={false} onclose={() => (open = false)}>
	<div class="mb-6 grid gap-6 md:grid-cols-2">
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Association</Label>
			<Select bind:value={dataPoint.associationId} placeholder="New Association" required>
				{#each association_states as association_state}
					<option value={association_state.association.id}>{association_state.association.name}</option>
				{/each}
			</Select>
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Data Point Name</Label>
			<Input type="text" bind:value={dataPoint.name} placeholder="Name" required />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Domain</Label>
			<Input type="text" bind:value={dataPoint.domain} placeholder="Domain (Optional)" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Data Type</Label>
			<Select type="text" bind:value={dataPoint.dataPointType} placeholder="Type" required>
				<option value="Real">Real</option>
				<option value="State">State</option>
				<option value="Discrete">Discrete</option>
				<option value="RealQ">RealQ</option>
				<option value="StateQ">StateQ</option>
				<option value="DiscreteQ">DiscreteQ</option>
			</Select>
		</div>
	</div>

	{#snippet footer()}
		<Button
			type="submit"
			color="blue"
			onclick={async () => {
				await context.getScadaForgeRequestService().createIccpDataPoint({
					associationId: dataPoint.associationId
				});
				dataPoint = createBlankEntry();
				open = false;
			}}>Create</Button
		>
		<Button
			color="alternative"
			onclick={() => {
				dataPoint = createBlankEntry();
				open = false;
			}}>Cancel</Button
		>
	{/snippet}
</Modal>
