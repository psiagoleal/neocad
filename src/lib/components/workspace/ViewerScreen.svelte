<script lang="ts">
	import ViewerDropzone from '$lib/components/workspace/ViewerDropzone.svelte';
	import ViewerToolbar from '$lib/components/workspace/ViewerToolbar.svelte';
	import type { CadViewerDocumentState, CadViewerProgressState } from '$lib/types/cad';

	type ViewerScreenProps = {
		currentDocument: CadViewerDocumentState | null;
		backgroundTheme: 'light' | 'dark';
		progress: CadViewerProgressState | null;
		isViewerReady: boolean;
		isOpening: boolean;
		isDragActive: boolean;
		isTauriRuntime: boolean;
		onOpenDrawing: () => void | Promise<void>;
		onFitView: () => void;
		onToggleBackground: () => void;
		onDragEnter: (event: DragEvent) => void;
		onDragOver: (event: DragEvent) => void;
		onDragLeave: (event: DragEvent) => void;
		onDrop: (event: DragEvent) => void | Promise<void>;
		onViewerHostReady: (element: HTMLDivElement) => void;
	};

	let {
		currentDocument,
		backgroundTheme,
		progress,
		isViewerReady,
		isOpening,
		isDragActive,
		isTauriRuntime,
		onOpenDrawing,
		onFitView,
		onToggleBackground,
		onDragEnter,
		onDragOver,
		onDragLeave,
		onDrop,
		onViewerHostReady
	}: ViewerScreenProps = $props();
</script>

<section class="viewer-frame card-panel viewer-focus-frame">
	<header class="viewer-header compact-header viewer-heading-row">
		<div>
			<p class="eyebrow">Canvas CAD</p>
			<h2>{currentDocument?.docTitle ?? 'Área principal de visualização'}</h2>
		</div>

		<ViewerToolbar
			hasDocument={currentDocument != null}
			{backgroundTheme}
			{isViewerReady}
			{isOpening}
			{onOpenDrawing}
			{onFitView}
			{onToggleBackground}
		/>
	</header>

	<div class="viewer-meta-strip viewer-meta-strip-compact">
		<div class="meta-pill">
			<span class="label">Modo</span>
			<strong>{currentDocument?.mode ?? 'Aguardando'}</strong>
		</div>
		<div class="meta-pill">
			<span class="label">Tema</span>
			<strong>{backgroundTheme === 'dark' ? 'Escuro' : 'Claro'}</strong>
		</div>
		{#if progress}
			<div class="progress-pill progress-pill-wide">
				<span class="label">Carregamento</span>
				<strong>{progress.percentage.toFixed(0)}%</strong>
				<span>{progress.stage}{progress.subStage ? ` / ${progress.subStage}` : ''}</span>
			</div>
		{/if}
	</div>

	<ViewerDropzone
		{currentDocument}
		{isDragActive}
		{isTauriRuntime}
		{onDragEnter}
		{onDragOver}
		{onDragLeave}
		{onDrop}
		onHostReady={onViewerHostReady}
	/>
</section>
