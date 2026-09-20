<script lang="ts">
	import { getApplicationContext } from '$lib/contexts/ApplicationContext';
	import type { AcseListener } from '$lib/services/ScadaForgeRequestService';
	import { Button, Heading, Input, Label, Modal, Select, Spinner } from 'flowbite-svelte';
	import { onMount } from 'svelte';

	const createBlankEntry = (): AcseListener => {
		return { id: '', name: '', host: '', port: 102 };
	};

	let context = getApplicationContext();

	let disabled = $state(false);
	let listener = $state(createBlankEntry());

	let { open = $bindable(false), initialValue = createBlankEntry(), onsuccess }: { open: boolean; initialValue: AcseListener | undefined; onsuccess: (listener: AcseListener) => void } = $props();

	$effect(() => {
		open; // Track open so this gets executed when the form is opened not just when the initial value changes.
		listener = { ...initialValue };
	});
	let successButtonName = $derived(!listener.id ? 'Create' : 'Update');
</script>

<Modal title="ACSE Listener" form class="border" {open} outsideclose={false} onclose={() => (open = false)}>
	<div class="mb-6 grid gap-6 md:grid-cols-2">
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Name</Label>
			<Input {disabled} type="text" bind:value={listener.name} required />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Type</Label>
			<Input disabled={true} type="text" value="TCP" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Domain</Label>
			<Input {disabled} type="text" value={listener.host} />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote Port</Label>
			<Input {disabled} type="number" min="1" max="65535" bind:value={() => listener.port, (v) => (listener.port = typeof v != 'number' || !Number.isFinite(v) ? listener.port : v < 1 ? listener.port : v > 65535 ? listener.port : v)} />
		</div>
	</div>

	{#snippet footer()}
		<Button
			{disabled}
			type="submit"
			color="blue"
			class="h-10 w-30"
			onclick={async () => {
				if (!!listener.id) {
					await context.getScadaForgeRequestService().createAcseListener({ ...listener, id: '' });
				} else {
					await context.getScadaForgeRequestService().updateAcseListener({ ...listener, id: '' });
				}
				open = false;
			}}><Spinner class="h-5 pl-1" type="bars" color="primary" /></Button
		>
		<Button
			{disabled}
			color="alternative"
			class="h-10 w-30"
			onclick={() => {
				open = false;
			}}><Spinner class="h-5 pl-1" type="bars" color="blue" /></Button
		>
	{/snippet}
</Modal>
