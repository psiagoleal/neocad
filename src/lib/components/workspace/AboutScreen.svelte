<script lang="ts">
	type AboutScreenProps = {
		appName: string;
		status: string;
		license: string;
		runtimeLabel: string;
		primaryStack: readonly string[];
		nextMilestones: readonly string[];
		supportedTargets: readonly string[];
	};

	let {
		appName,
		status,
		license,
		runtimeLabel,
		primaryStack,
		nextMilestones,
		supportedTargets
	}: AboutScreenProps = $props();
</script>

<div class="about-grid">
	<section class="home-panel card-panel">
		<p class="eyebrow">Sobre o app</p>
		<h2>Detalhes de planejamento e implementação</h2>
		<p class="support-copy compact-copy">
			Esta área concentra informações institucionais e técnicas para que a tela principal fique mais
			limpa e focada no trabalho com o desenho.
		</p>
		<ul class="details-list">
			<li><span>Aplicativo</span><strong>{appName}</strong></li>
			<li><span>Status</span><strong>{status}</strong></li>
			<li><span>Runtime atual</span><strong>{runtimeLabel}</strong></li>
			<li><span>Licença</span><strong>{license}</strong></li>
		</ul>
	</section>

	<section class="home-panel card-panel">
		<h2>Stack principal</h2>
		<ul class="details-list compact-list">
			{#each primaryStack as item (item)}
				<li><span>Stack</span><strong>{item}</strong></li>
			{/each}
		</ul>
	</section>

	<section class="home-panel card-panel">
		<h2>Próximos passos</h2>
		<ul class="plain-list">
			{#each nextMilestones as milestone (milestone)}
				<li>{milestone}</li>
			{/each}
		</ul>
	</section>

	<section class="home-panel card-panel">
		<h2>Plataformas suportadas</h2>
		<div class="chips">
			{#each supportedTargets as target (target)}
				<span class="status-chip">{target}</span>
			{/each}
		</div>
	</section>
</div>
