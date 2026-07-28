<script lang="ts">
	import { getApplicationContext } from '$lib/contexts/ApplicationContext';
	import { Button, Heading, Input, Label, Modal } from 'flowbite-svelte';

	const createBlankEntry = () => {
		return {
			associationName: '',
			associationType: 'Client',
			associationHost: '',
			associationPort: 102,
			associationLocalApTitle: '',
			associationRemoteApTitle: '',
			associationLocalAeQualifier: '',
			associationRemoteAeQualifier: '',
			associationLocalTsap: '',
			associationRemoteTsap: '',
			associationLocalSsap: '',
			associationRemoteSsap: '',
			associationLocalPsap: '',
			associationRemotePsap: ''
		};
	};

	let { open = $bindable(false) } = $props();

	let context = getApplicationContext();
	let association = $state(createBlankEntry());
</script>

<Modal title="New TASE.2 Initiator" form class="border" {open} outsideclose={false} onclose={() => (open = false)}>
	<div class="mb-6 grid gap-6 md:grid-cols-2">
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Name</Label>
			<Input type="text" bind:value={association.associationName} placeholder="New Association" required />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Type</Label>
			<Input type="text" value={association.associationType} disabled />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote Host</Label>
			<Input type="text" bind:value={association.associationHost} placeholder="IP Address or Hostname" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote Port</Label>
			<Input
				type="number"
				min="1"
				max="65535"
				bind:value={
					() => association.associationPort,
					(v) => (association.associationPort = typeof v != 'number' || !Number.isFinite(v) ? association.associationPort : v < 1 ? association.associationPort : v > 65535 ? association.associationPort : v)
				}
			/>
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Local AP Title</Label>
			<Input type="text" bind:value={association.associationLocalApTitle} placeholder="1.2.3.4" pattern="([0-9]+)(\.([0-9]+))*" required></Input>
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote AP Title</Label>
			<Input type="text" bind:value={association.associationRemoteApTitle} placeholder="1.2.3.4" pattern="([0-9]+)(\.([0-9]+))*" required></Input>
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Local AE Qualifier</Label>
			<Input type="text" bind:value={association.associationLocalAeQualifier} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" required />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote AE Qualifier</Label>
			<Input type="text" bind:value={association.associationRemoteAeQualifier} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" required />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Local TSAP</Label>
			<Input type="text" bind:value={association.associationLocalTsap} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote TSAP</Label>
			<Input type="text" bind:value={association.associationRemoteTsap} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Local SSAP</Label>
			<Input type="text" bind:value={association.associationLocalSsap} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote SSAP</Label>
			<Input type="text" bind:value={association.associationRemoteSsap} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Local PSAP</Label>
			<Input type="text" bind:value={association.associationLocalPsap} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" />
		</div>
		<div>
			<Label class="text-heading mb-2.5 block text-sm font-medium">Remote PSAP</Label>
			<Input type="text" bind:value={association.associationRemotePsap} placeholder="Hexadecimal Bytes: 0015ABCD" pattern="([0-9a-zA-Z][0-9a-zA-Z])+" />
		</div>
		<div class="col-span-2">
			<Label class="text-heading mb-2.5 block text-sm font-medium">Authentication</Label>
			<Input type="text" value="None" disabled />
		</div>
	</div>

	{#snippet footer()}
		<Button
			type="submit"
			color="primary"
			onclick={() => {
				context.getScadaForgeStreamService()
				association = createBlankEntry();
				open = false;
			}}>Create</Button
		>
		<Button
			color="alternative"
			onclick={() => {
				association = createBlankEntry();
				open = false;
			}}>Cancel</Button
		>
	{/snippet}
</Modal>
