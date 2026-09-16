<script lang="ts">
	import { getApplicationContext } from '$lib/contexts/ApplicationContext';
	import { Button, Heading, Input, Label, Modal, Select } from 'flowbite-svelte';

	const createBlankEntry = () => {
		return {
			id: '',
			name: '',
			host: '',
			port: 102,
		};
	};

	let { open = $bindable(false) } = $props();

	let context = getApplicationContext();
	let listener = $state(createBlankEntry());
</script>

<Modal title="ACSE Listener" form class="border" {open} outsideclose={false} onclose={() => (open = false)}>
	<div class="mb-6 grid gap-6 md:grid-cols-2">
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Name</Label>
			<Input type="text" bind:value={listener.name} placeholder="New Listener" required />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Type</Label>
			<Input type="text" value="TCP" disabled />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Domain</Label>
			<Input type="text" value={listener.host} />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote Port</Label>
			<Input
				type="number"
				min="1"
				max="65535"
				bind:value={
					() => listener.port,
					(v) => (listener.port = typeof v != 'number' || !Number.isFinite(v) ? listener.port : v < 1 ? listener.port : v > 65535 ? listener.port : v)
				}
			/>
		</div>
	</div>

	{#snippet footer()}
		<Button
			type="submit"
			color="blue"
			onclick={async () => {
				await context.getScadaForgeRequestService().createAcseListener({
					id: '',
					name: listener.name,
					host: listener.host,
					port: listener.port,
				});
				listener = createBlankEntry();
				open = false;
			}}>Create</Button
		>
		<Button
			color="alternative"
			onclick={() => {
				listener = createBlankEntry();
				open = false;
			}}>Cancel</Button
		>
	{/snippet}
</Modal>
