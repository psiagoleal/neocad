<!-- Caminho relativo: src/routes/+layout.svelte -->
<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { appMetadata } from '$lib/config/app';
	import '$lib/styles/index.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>{appMetadata.name}</title>
	<meta name="description" content={appMetadata.tagline} />
</svelte:head>

{@render children()}
