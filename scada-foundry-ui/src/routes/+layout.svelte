<script lang="ts">
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import favicon from '$lib/assets/favicon.svg';
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
	import { ApplicationContext } from '$lib/contexts/ApplicationContext';

	let { children } = $props();

	export const [getApplicationContext, setApplicationContext] = createContext<ApplicationContext>();

	let connectionStatus = $state('Idle');

	onMount(() => {
		setApplicationContext(new ApplicationContext());
		getApplicationContext().getScadaForgeStreamService().addListener((event) => {
			connectionStatus = 'Connected';
		});
	});

	let activeUrl = $state(page.url.pathname);
	const spanClass = 'flex-1 ms-3 whitespace-nowrap';

	const sidebarMatch: string | string[] = 'docs/components/sidebar';
	const matchesRoute = $derived.by(() => {
		const list = Array.isArray(sidebarMatch) ? sidebarMatch : [sidebarMatch];
		return list.some((p) => activeUrl.startsWith(`/${p}`));
	});

	$effect(() => {
		activeUrl = page.url.pathname;
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<header>
	<Navbar
		fluid={true}
		class="z-20 h-17 border-b border-gray-200 bg-white dark:border-gray-600 dark:bg-gray-800"
	>
		<NavBrand href="/">
			<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="48" height="48">
				<defs>
					<!-- Background Gradient (Charcoal) -->
					<linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
						<stop offset="0%" stop-color="#121212" />
						<stop offset="100%" stop-color="#2a2a2a" />
					</linearGradient>

					<!-- Foundry Core Gradient (Hot Forge / Fire) -->
					<linearGradient id="coreGrad" x1="0%" y1="0%" x2="100%" y2="100%">
						<stop offset="0%" stop-color="#f97316" />
						<stop offset="100%" stop-color="#b91c1c" />
					</linearGradient>

					<!-- Signal Gradients for Data Flowing In -->
					<linearGradient id="line1" x1="0%" y1="0%" x2="100%" y2="100%">
						<stop offset="0%" stop-color="#38bdf8" />
						<stop offset="100%" stop-color="#f97316" />
					</linearGradient>
					<linearGradient id="line2" x1="0%" y1="100%" x2="100%" y2="0%">
						<stop offset="0%" stop-color="#a855f7" />
						<stop offset="100%" stop-color="#f97316" />
					</linearGradient>
					<linearGradient id="line3" x1="100%" y1="100%" x2="0%" y2="0%">
						<stop offset="0%" stop-color="#10b981" />
						<stop offset="100%" stop-color="#f97316" />
					</linearGradient>
					<linearGradient id="line4" x1="100%" y1="0%" x2="0%" y2="100%">
						<stop offset="0%" stop-color="#facc15" />
						<stop offset="100%" stop-color="#f97316" />
					</linearGradient>

					<!-- Glowing Effect for the Foundry Core -->
					<!-- Increased bounds from -20% to -100% to prevent hard squared-off edges on smaller shapes like circles -->
					<filter id="glow" x="-100%" y="-100%" width="300%" height="300%">
						<feGaussianBlur stdDeviation="12" result="blur" />
						<feMerge>
							<feMergeNode in="blur" />
							<feMergeNode in="SourceGraphic" />
						</feMerge>
					</filter>
				</defs>

				<!-- Background Canvas -->
				<rect width="512" height="512" rx="96" fill="url(#bg)" />

				<!-- Flowing Data Lines from Points to the Core Hub -->
				<g stroke-width="14" stroke-linecap="round" fill="none">
					<!-- Top Left -->
					<path d="M 128 128 Q 256 128 256 256" stroke="url(#line1)" />
					<!-- Bottom Left -->
					<path d="M 128 384 Q 128 256 256 256" stroke="url(#line2)" />
					<!-- Bottom Right -->
					<path d="M 384 384 Q 256 384 256 256" stroke="url(#line3)" />
					<!-- Top Right -->
					<path d="M 384 128 Q 384 256 256 256" stroke="url(#line4)" />
				</g>

				<!-- Outer Protocol Nodes (Connecting Points) -->
				<g>
					<circle cx="128" cy="128" r="24" fill="#38bdf8" />
					<circle cx="128" cy="128" r="10" fill="#121212" />

					<circle cx="128" cy="384" r="24" fill="#a855f7" />
					<circle cx="128" cy="384" r="10" fill="#121212" />

					<circle cx="384" cy="384" r="24" fill="#10b981" />
					<circle cx="384" cy="384" r="10" fill="#121212" />

					<circle cx="384" cy="128" r="24" fill="#facc15" />
					<circle cx="384" cy="128" r="10" fill="#121212" />
				</g>

				<!-- Multi-layered Foundry Core (Hexagon Forge + Data Point) -->
				<g filter="url(#glow)">
					<!-- Base Hexagon -->
					<polygon points="256,160 339,208 339,304 256,352 173,304 173,208" fill="url(#coreGrad)" />
					<!-- Inner Dark Hexagon to create a ring effect -->
					<polygon points="256,180 322,218 322,294 256,332 190,294 190,218" fill="#1a1a1a" />
				</g>

				<!-- Dead Center Data Node -->
				<circle cx="256" cy="256" r="28" fill="#f97316" filter="url(#glow)" />
				<circle cx="256" cy="256" r="14" fill="#fff" />
			</svg>

			<span class="ml-4 self-center text-xl font-semibold whitespace-nowrap dark:text-white"
				>SCADA Foundry</span
			><span
				class="ml-4 self-center text-xl font-semibold whitespace-nowrap text-green-500 drop-shadow-[0_0_5px_rgba(0,255,0,1)]"
				>⬤</span
			><Popover class="w-64 text-sm" title="Client Status"
				>Not connected to server. Retrying automatically.</Popover
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
					<SidebarItem label="Overview" href={resolve('/')}>
						{#snippet icon()}
							<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512" width="16" height="16">
								<defs>
									<!-- Foundry Core Gradient (Hot Forge / Fire) -->
									<linearGradient id="coreGrad" x1="0%" y1="0%" x2="100%" y2="100%">
										<stop offset="0%" stop-color="#f97316" />
										<stop offset="100%" stop-color="#b91c1c" />
									</linearGradient>

									<!-- Signal Gradients for Data Flowing In -->
									<linearGradient id="line1" x1="0%" y1="0%" x2="100%" y2="100%">
										<stop offset="0%" stop-color="#38bdf8" />
										<stop offset="100%" stop-color="#f97316" />
									</linearGradient>
									<linearGradient id="line2" x1="0%" y1="100%" x2="100%" y2="0%">
										<stop offset="0%" stop-color="#a855f7" />
										<stop offset="100%" stop-color="#f97316" />
									</linearGradient>
									<linearGradient id="line3" x1="100%" y1="100%" x2="0%" y2="0%">
										<stop offset="0%" stop-color="#10b981" />
										<stop offset="100%" stop-color="#f97316" />
									</linearGradient>
									<linearGradient id="line4" x1="100%" y1="0%" x2="0%" y2="100%">
										<stop offset="0%" stop-color="#facc15" />
										<stop offset="100%" stop-color="#f97316" />
									</linearGradient>

									<!-- Glowing Effect for the Foundry Core -->
									<filter id="glow" x="-100%" y="-100%" width="300%" height="300%">
										<feGaussianBlur stdDeviation="12" result="blur" />
										<feMerge>
											<feMergeNode in="blur" />
											<feMergeNode in="SourceGraphic" />
										</feMerge>
									</filter>

									<!-- Mask to punch transparent holes where the dark background used to show through -->
									<mask id="cutout">
										<rect width="512" height="512" fill="white" />
										<circle cx="128" cy="128" r="10" fill="black" />
										<circle cx="128" cy="384" r="10" fill="black" />
										<circle cx="384" cy="384" r="10" fill="black" />
										<circle cx="384" cy="128" r="10" fill="black" />
										<polygon
											points="256,180 322,218 322,294 256,332 190,294 190,218"
											fill="black"
										/>
									</mask>
								</defs>

								<!-- EVERYTHING overlapping the holes is wrapped in the mask so underlying lines get cut -->
								<g mask="url(#cutout)">
									<!-- Flowing Data Lines from Points to the Core Hub -->
									<g stroke-width="14" stroke-linecap="round" fill="none">
										<path d="M 128 128 Q 256 128 256 256" stroke="url(#line1)" />
										<path d="M 128 384 Q 128 256 256 256" stroke="url(#line2)" />
										<path d="M 384 384 Q 256 384 256 256" stroke="url(#line3)" />
										<path d="M 384 128 Q 384 256 256 256" stroke="url(#line4)" />
									</g>

									<!-- Outer Nodes -->
									<circle cx="128" cy="128" r="24" fill="#38bdf8" />
									<circle cx="128" cy="384" r="24" fill="#a855f7" />
									<circle cx="384" cy="384" r="24" fill="#10b981" />
									<circle cx="384" cy="128" r="24" fill="#facc15" />

									<!-- Base Hexagon -->
									<polygon
										points="256,160 339,208 339,304 256,352 173,304 173,208"
										fill="url(#coreGrad)"
										filter="url(#glow)"
									/>
								</g>

								<!-- Dead Center Data Node -->
								<circle cx="256" cy="256" r="28" fill="#f97316" filter="url(#glow)" />
								<circle cx="256" cy="256" r="14" fill="#fff" />
							</svg>
						{/snippet}
					</SidebarItem>
					<SidebarDropdownWrapper label="ICCP" classes={{ btn: 'p-2' }} isOpen={matchesRoute}>
						{#snippet icon()}
							<ShareNodesOutline
								class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
							/>
						{/snippet}
						<SidebarItem label="Associations" href="/iccp/associations" />
						<SidebarItem label="Scan" href="/iccp/scan" />
						<SidebarItem label="Scan" href="/iccp/datasets" />
					</SidebarDropdownWrapper>
					<SidebarDropdownWrapper label="DNP3" classes={{ btn: 'p-2' }} isOpen={matchesRoute}>
						{#snippet icon()}
							<CodeMergeOutline
								class="h-5 w-5 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white"
							/>
						{/snippet}
						<SidebarItem label="Coming Soon" href="/" />
					</SidebarDropdownWrapper>
				</SidebarGroup>
			</Sidebar>
		</div>
		<div class="z-100 flex-1 overflow-auto">
			{@render children()}
		</div>
	</div>
</div>
