<!-- Caminho relativo: src/routes/+page.svelte -->
<script lang="ts">
	import { appMetadata, nextMilestones, primaryStack, supportedTargets } from '$lib/config/app';

	const highlights = [
		'Base SvelteKit criada com TypeScript, ESLint, Prettier, Vitest e Playwright.',
		'Projeto preparado para modo SPA com adapter-static, compatível com Tauri.',
		'Documentação de arquitetura e desenvolvimento criada desde a Fase 0.'
	] as const;
</script>

<svelte:head>
	<title>{appMetadata.name} — Fase 1</title>
	<meta
		name="description"
		content="Scaffold inicial do NeoCAD com SvelteKit, Svelte 5 e preparação para Tauri 2."
	/>
</svelte:head>

<section class="page-shell">
	<div class="hero">
		<p class="eyebrow">Fase 1 — Bootstrap do aplicativo</p>
		<h1>{appMetadata.name}</h1>
		<p class="tagline">{appMetadata.tagline}</p>
		<p class="summary">
			O frontend base já está pronto para a próxima etapa: inicialização do shell desktop com Tauri
			2 e integração gradual com o <code>cad-viewer</code>.
		</p>

		<div class="cta-row">
			<a
				class="primary-link"
				href="https://github.com/mlightcad/cad-viewer"
				target="_blank"
				rel="noreferrer"
			>
				Ver upstream cad-viewer
			</a>
			<span class="status-chip">{appMetadata.status}</span>
		</div>
	</div>

	<div class="grid">
		<article class="card">
			<h2>Stack base</h2>
			<ul>
				{#each primaryStack as item (item)}
					<li>{item}</li>
				{/each}
			</ul>
		</article>

		<article class="card">
			<h2>Plataformas alvo</h2>
			<ul>
				{#each supportedTargets as target (target)}
					<li>{target}</li>
				{/each}
			</ul>
		</article>

		<article class="card">
			<h2>Entregas desta fase</h2>
			<ul>
				{#each highlights as highlight (highlight)}
					<li>{highlight}</li>
				{/each}
			</ul>
		</article>

		<article class="card card-wide">
			<h2>Próximos passos imediatos</h2>
			<ol>
				{#each nextMilestones as milestone (milestone)}
					<li>{milestone}</li>
				{/each}
			</ol>
		</article>
	</div>
</section>

<style>
	:global(body) {
		margin: 0;
		min-height: 100vh;
		font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
		background:
			radial-gradient(circle at top, rgba(78, 143, 255, 0.24), transparent 32%),
			linear-gradient(180deg, #081121 0%, #091728 55%, #0d1f35 100%);
		color: #edf4ff;
	}

	.page-shell {
		max-width: 1120px;
		margin: 0 auto;
		padding: 4rem 1.5rem 5rem;
	}

	.hero {
		display: grid;
		gap: 1rem;
		margin-bottom: 2rem;
	}

	.eyebrow {
		margin: 0;
		text-transform: uppercase;
		letter-spacing: 0.14em;
		font-size: 0.78rem;
		font-weight: 700;
		color: #7fb3ff;
	}

	h1 {
		margin: 0;
		font-size: clamp(2.8rem, 8vw, 5rem);
		line-height: 0.95;
	}

	.tagline,
	.summary {
		margin: 0;
		max-width: 60rem;
		font-size: 1.08rem;
		line-height: 1.7;
		color: #c9daf8;
	}

	.cta-row {
		display: flex;
		flex-wrap: wrap;
		gap: 0.9rem;
		align-items: center;
	}

	.primary-link,
	.status-chip {
		border-radius: 999px;
		padding: 0.8rem 1.1rem;
		font-weight: 700;
		text-decoration: none;
	}

	.primary-link {
		background: #7fb3ff;
		color: #081121;
	}

	.primary-link:hover {
		background: #9fc5ff;
	}

	.status-chip {
		background: rgba(127, 179, 255, 0.12);
		border: 1px solid rgba(127, 179, 255, 0.25);
		color: #dbe9ff;
	}

	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
		gap: 1rem;
	}

	.card {
		padding: 1.25rem;
		border-radius: 1rem;
		background: rgba(10, 25, 44, 0.68);
		border: 1px solid rgba(127, 179, 255, 0.16);
		backdrop-filter: blur(10px);
		box-shadow: 0 12px 36px rgba(0, 0, 0, 0.18);
	}

	.card-wide {
		grid-column: 1 / -1;
	}

	h2 {
		margin-top: 0;
		margin-bottom: 1rem;
		font-size: 1.05rem;
	}

	ul,
	ol {
		margin: 0;
		padding-left: 1.2rem;
		color: #dce9ff;
		line-height: 1.7;
	}

	li + li {
		margin-top: 0.45rem;
	}

	code {
		padding: 0.15rem 0.35rem;
		border-radius: 0.4rem;
		background: rgba(255, 255, 255, 0.09);
		font-size: 0.95em;
	}

	@media (max-width: 640px) {
		.page-shell {
			padding-top: 3rem;
		}
	}
</style>
