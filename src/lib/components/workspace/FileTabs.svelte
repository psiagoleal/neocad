<!-- Caminho relativo: src/lib/components/workspace/FileTabs.svelte -->

<!--
	Abas de arquivo, como no AutoCAD: `Início` e o desenho aberto (spec 11, I2).
	Com um desenho só, a aba dele não fecha.
-->
<script lang="ts">
	type FileTabsProps = {
		/** Nome do desenho da aba, ou `null` enquanto o primeiro não existe. */
		drawingTitle: string | null;
		/** A aba ativa: a do desenho ou `Início`. */
		active: 'home' | 'drawing' | null;
		onShowHome: () => void;
		onShowDrawing: () => void;
	};

	let { drawingTitle, active, onShowHome, onShowDrawing }: FileTabsProps = $props();
</script>

<div class="file-tabs" role="tablist" aria-label="Desenhos abertos">
	<button
		class="file-tab"
		class:file-tab-active={active === 'home'}
		type="button"
		role="tab"
		aria-selected={active === 'home'}
		onclick={onShowHome}
	>
		Início
	</button>
	{#if drawingTitle != null}
		<button
			class="file-tab"
			class:file-tab-active={active === 'drawing'}
			type="button"
			role="tab"
			aria-selected={active === 'drawing'}
			onclick={onShowDrawing}
		>
			{drawingTitle}
		</button>
	{/if}
</div>

<style>
	.file-tabs {
		display: flex;
		gap: 2px;
		min-width: 0;
		overflow-x: auto;
		border-bottom: 1px solid var(--color-border);
	}

	.file-tab {
		padding: 0.35rem 0.9rem;
		border: 1px solid transparent;
		border-bottom: none;
		border-radius: var(--radius-sm, 4px) var(--radius-sm, 4px) 0 0;
		background: var(--color-surface-muted);
		color: var(--color-text-soft);
		font: inherit;
		font-size: 0.85rem;
		white-space: nowrap;
		cursor: pointer;
	}

	.file-tab:hover {
		color: var(--color-text);
	}

	.file-tab-active {
		background: var(--color-surface-strong);
		border-color: var(--color-border-strong);
		color: var(--color-text);
	}
</style>
