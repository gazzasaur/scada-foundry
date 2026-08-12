<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
	import scadaFoundryIcon from '$lib/assets/scada-foundry-icon.svg';
	import scadaFoundryButtonIcon from '$lib/assets/scada-foundry-icon-transparent.svg';
	import {
		Navbar,
		NavBrand,
		Popover,
		Sidebar,
		SidebarDropdownWrapper,
		SidebarGroup,
		SidebarItem
	} from 'flowbite-svelte';
	import './layout.css';
	import { onMount } from 'svelte';
	import { CodeMergeOutline, ShareNodesOutline } from 'flowbite-svelte-icons';
	import { createContext } from 'svelte';
	import {
		ApplicationContext,
		getApplicationContext,
		setApplicationContext
	} from '$lib/contexts/ApplicationContext';

	let { children } = $props();

	let connectionStatus = $state({
		state: 'Idle',
		kind: 'ScadaForgeStatus',
		message: 'Client is starting. It will connect shortly.'
	});

	setApplicationContext(new ApplicationContext());
	getApplicationContext()
		.getScadaForgeStreamService()
		.addListener((event) => {
			if (event.kind == 'ScadaForgeStatus') {
				connectionStatus = event;
			}
		});

	let activeUrl = $derived(page.url.pathname);

	const sidebarMatch: string | string[] = 'docs/components/sidebar';
	const matchesRoute = $derived.by(() => {
		const list = Array.isArray(sidebarMatch) ? sidebarMatch : [sidebarMatch];
		return list.some((p) => activeUrl.startsWith(`/${p}`));
	});

	let statusColour = $derived.by(() => {
		switch (connectionStatus.state) {
			case 'Idle':
				return 'text-blue-500';
			case 'Connected':
				return 'text-green-500';
			case 'Failed':
				return 'text-red-500';
			default:
				return 'text-blue-500';
		}
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<header>
	<Navbar
		fluid={true}
		class="z-20 h-17 border-b border-gray-200 bg-white dark:border-gray-600 dark:bg-gray-800"
	>
		<NavBrand href="/">
			<img src={scadaFoundryIcon} class="w-12" alt="Scada Foundry Icon" /><span
				class="ml-4 self-center text-xl font-semibold whitespace-nowrap dark:text-white"
				>SCADA Foundry</span
			><span
				class="ml-4 self-center text-xl font-semibold whitespace-nowrap {statusColour} drop-shadow-[0_0_5px_rgba(255,255,255,1)]"
				>⬤</span
			><Popover class="w-64 text-sm" title="Client Status: {connectionStatus.state}"
				>{connectionStatus.message}</Popover
			>
		</NavBrand>
	</Navbar>
</header>

<div class="fixed top-0 z-10 block h-dvh w-full bg-white pt-17 dark:bg-gray-800">
	<div class="flex h-full w-full">
		<div>
			<Sidebar
				class="relative block h-full border-r border-gray-200 dark:border-gray-600"
				{activeUrl}
				isOpen={true}
				backdrop={false}
				isSingle={false}
			>
				<SidebarGroup>
					<SidebarItem label="Overview" class="mt-1 mb-1" href={resolve('/')}>
						{#snippet icon()}
							<img class="w-5" src={scadaFoundryButtonIcon} alt="SCADA Foundry Button" />
						{/snippet}
					</SidebarItem>
					<SidebarDropdownWrapper label="ICCP" classes={{ btn: 'p-2' }} isOpen={matchesRoute}>
						{#snippet icon()}
							<ShareNodesOutline
								class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
							/>
						{/snippet}
						<SidebarItem label="Associations" class="mt-1 mb-1" href={resolve("/iccp/associations")} />
						<SidebarItem label="Data Points" class="mt-1 mb-1" href={resolve("/iccp/datapoints")} />
					</SidebarDropdownWrapper>
					<SidebarDropdownWrapper label="DNP3" classes={{ btn: 'p-2' }} isOpen={matchesRoute}>
						{#snippet icon()}
							<CodeMergeOutline
								class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
							/>
						{/snippet}
						<SidebarItem label="Coming Soon" class="mt-1 mb-1" href={resolve("/dnp3")} />
					</SidebarDropdownWrapper>
				</SidebarGroup>
			</Sidebar>
		</div>
		<div class="z-100 flex-1 overflow-auto">
			{@render children()}
		</div>
	</div>
</div>
