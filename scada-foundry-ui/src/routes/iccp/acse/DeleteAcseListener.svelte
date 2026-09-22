<script lang="ts">
	import { ApplicationContext, getApplicationContext } from '$lib/contexts/ApplicationContext';
	import type { AcseListener, AcseListenerReport } from '$lib/services/ScadaForgeRequestService';
	import { Button, Heading, Input, Label, Modal, Select } from 'flowbite-svelte';

	let context = getApplicationContext();
	let { open = $bindable(false), existing = $bindable() } = $props();
</script>

<Modal title="ACSE Listener" form class="border" {open} outsideclose={false} onclose={() => (open = false)}>
	<Label class="text-heading mb-2.5 block text-sm font-medium">Are you sure you would like to delete listener?</Label>

	{#snippet footer()}
		<Button
			type="submit"
			color="blue"
			onclick={async () => {
				await context.getScadaForgeRequestService().deleteAcseListener(existing.id);
				open = false;
			}}>Confirm</Button
		>
		<Button color="alternative" onclick={() => (open = false)}>Cancel</Button>
	{/snippet}
</Modal>
