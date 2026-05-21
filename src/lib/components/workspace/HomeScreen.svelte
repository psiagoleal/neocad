<script lang="ts">
	import RecentDocumentsPanel from '$lib/components/workspace/RecentDocumentsPanel.svelte';
	import type { CadRecentDocument } from '$lib/types/cad';

	type HomeScreenProps = {
		runtimeLabel: string;
		isViewerReady: boolean;
		isMessagesVisible: boolean;
		isOpening: boolean;
		isTauriRuntime: boolean;
		recentDocuments: CadRecentDocument[];
		onOpenDrawing: () => void | Promise<void>;
		onEnterViewer: () => void;
		onOpenRecent: (recentDocument: CadRecentDocument) => void | Promise<void>;
		onClearRecents: () => void | Promise<void>;
	};

	let {
		runtimeLabel,
		isViewerReady,
		isMessagesVisible,
		isOpening,
		isTauriRuntime,
		recentDocuments,
		onOpenDrawing,
		onEnterViewer,
		onOpenRecent,
		onClearRecents
	}: HomeScreenProps = $props();

	function handleOpenDrawing(): void {
		void onOpenDrawing();
	}
</script>

<div class="home-grid">
	<section class="home-hero card-panel">
		<p class="eyebrow">Fluxo principal</p>
		<h2>Abra um desenho e entre no workspace principal do NeoCAD</h2>
		<p class="support-copy compact-copy">
			Abra um desenho para entrar direto no canvas. Informações de planejamento e detalhes de
			implementação ficam disponíveis separadamente em <strong>Sobre</strong>.
		</p>

		<div class="hero-actions">
			<button
				class="primary-button"
				type="button"
				onclick={handleOpenDrawing}
				disabled={!isViewerReady || isOpening}
			>
				{isOpening ? 'Abrindo desenho...' : 'Abrir desenho CAD'}
			</button>
			<button class="secondary-button" type="button" onclick={onEnterViewer}
				>Ir para o canvas</button
			>
		</div>

		<div class="home-status-row" aria-label="Resumo do estado inicial do workspace">
			<span class="status-chip">{isViewerReady ? 'Viewer pronto' : 'Viewer inicializando'}</span>
			<span class="status-chip">Runtime {runtimeLabel}</span>
			<span class="status-chip"
				>{isMessagesVisible ? 'Mensagens visíveis' : 'Mensagens recolhidas'}</span
			>
		</div>

		<div class="home-notes">
			<p class="support-copy compact-copy">
				Use o menu <strong>Arquivo</strong> ou arraste um arquivo diretamente para o canvas para começar.
			</p>
			<p class="support-copy compact-copy">
				{isTauriRuntime
					? 'No runtime Tauri, os desenhos recentes podem ser reabertos por caminho completo.'
					: 'No navegador, a abertura funciona por fallback local sem reabertura por caminho persistido.'}
			</p>
		</div>
	</section>

	<RecentDocumentsPanel
		{recentDocuments}
		{isTauriRuntime}
		{isOpening}
		{onOpenRecent}
		{onClearRecents}
	/>
</div>
