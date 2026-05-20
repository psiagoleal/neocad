<script lang="ts">
	import RecentDocumentsPanel from '$lib/components/workspace/RecentDocumentsPanel.svelte';
	import type { CadRecentDocument } from '$lib/types/cad';

	type HomeScreenProps = {
		runtimeLabel: string;
		isViewerReady: boolean;
		isMessagesVisible: boolean;
		isOpening: boolean;
		isTauriRuntime: boolean;
		backgroundTheme: 'light' | 'dark';
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
		backgroundTheme,
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

		<div class="meta-grid">
			<div>
				<span class="label">Viewer</span>
				<strong>{isViewerReady ? 'Pronto' : 'Inicializando'}</strong>
			</div>
			<div>
				<span class="label">Runtime</span>
				<strong>{runtimeLabel}</strong>
			</div>
			<div>
				<span class="label">Mensagens</span>
				<strong>{isMessagesVisible ? 'Dock aberto' : 'Dock recolhido'}</strong>
			</div>
			<div>
				<span class="label">Canvas</span>
				<strong>{backgroundTheme === 'dark' ? 'Escuro' : 'Claro'}</strong>
			</div>
		</div>

		<ul class="plain-list compact-listing">
			<li>Use arrastar e soltar diretamente sobre o canvas para abrir DWG ou DXF.</li>
			<li>O fluxo atual prioriza o viewer e deixa informações institucionais na tela Sobre.</li>
			<li>
				{isTauriRuntime
					? 'No runtime Tauri, os recentes podem ser reabertos por caminho completo.'
					: 'No navegador, o fluxo mantém fallback local sem reabertura por caminho persistido.'}
			</li>
		</ul>
	</section>

	<RecentDocumentsPanel
		{recentDocuments}
		{isTauriRuntime}
		{isOpening}
		{onOpenRecent}
		{onClearRecents}
	/>
</div>
