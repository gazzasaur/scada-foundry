<script lang="ts">
	import { ApplicationContext, getApplicationContext } from '$lib/contexts/ApplicationContext';
	import type { AcseListener, AcseListenerReport } from '$lib/services/ScadaForgeRequestService';
	import { Button, Heading, Input, Label, Modal, Select } from 'flowbite-svelte';

	const createBlankEntry = () => {
		return { id: '', name: '', host: '0.0.0.0', port: 102 };
	};
	async function createOrUpdate(context: ApplicationContext, value: AcseListener) {
		if (!!value.id) {
			return await context.getScadaForgeRequestService().updateAcseListener(value);
		}
		return await context.getScadaForgeRequestService().createAcseListener(value);
	}

	let processing =  $state(false);
	let context = getApplicationContext();
	let listener = $state(createBlankEntry());
	let { open = $bindable(false), existing = undefined, onsuccess = (value: AcseListenerReport) => {} } = $props();

	$effect(() => {
		open; // Update using existing value when the dialog is open.
		if (!!existing) {
			listener = Object.assign({}, existing);
		}
	});
	let submitButtonLabel = $derived((!!listener.id) ? 'Update' : 'Create');
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
			<Label class="text-heading mb-2.5 block text-sm font-medium">Host</Label>
			<Input type="text" value="0.0.0.0" disabled />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote Port</Label>
			<Input type="number" min="1" max="65535" bind:value={() => listener.port, (v) => (listener.port = typeof v != 'number' || !Number.isFinite(v) ? listener.port : v < 1 ? listener.port : v > 65535 ? listener.port : v)} />
		</div>
	</div>

	{#snippet footer()}
		<Button
			type="submit"
			color="blue"
			onclick={async () => {
				let value = await createOrUpdate(context, listener);
				listener = createBlankEntry();
				onsuccess(value);
				open = false;
			}}>{submitButtonLabel}</Button
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
