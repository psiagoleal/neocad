<script lang="ts">
	type ViewerToolbarProps = {
		hasDocument: boolean;
		backgroundTheme: 'light' | 'dark';
		isViewerReady: boolean;
		isOpening: boolean;
		onOpenDrawing: () => void | Promise<void>;
		onFitView: () => void;
		onToggleBackground: () => void;
	};

	let {
		hasDocument,
		backgroundTheme,
		isViewerReady,
		isOpening,
		onOpenDrawing,
		onFitView,
		onToggleBackground
	}: ViewerToolbarProps = $props();

	function handleOpenDrawing(): void {
		void onOpenDrawing();
	}
</script>

<div class="viewer-toolbar">
	<button class="secondary-button" type="button" onclick={onFitView} disabled={!hasDocument}>
		Ajustar
	</button>
	<button
		class="secondary-button"
		type="button"
		onclick={onToggleBackground}
		disabled={!isViewerReady}
	>
		Fundo {backgroundTheme === 'dark' ? 'escuro' : 'claro'}
	</button>
	<button
		class="secondary-button"
		type="button"
		onclick={handleOpenDrawing}
		disabled={!isViewerReady || isOpening}
	>
		{isOpening ? 'Abrindo...' : 'Abrir'}
	</button>
</div>
