<!-- Caminho relativo: src/lib/components/workspace/ViewerScreen.svelte -->
<script lang="ts">
	import ViewerDropzone from '$lib/components/workspace/ViewerDropzone.svelte';
	import ViewerToolbar from '$lib/components/workspace/ViewerToolbar.svelte';
	import type { CadViewerDocumentState, CadViewerProgressState } from '$lib/types/cad';
	import type { TelaDoKernel } from '$lib/render/tela-do-kernel';

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
		isKernelScreenVisible: boolean;
		onKernelScreenReady: (tela: TelaDoKernel) => void;
		onKernelScreenError: (message: string) => void;
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
		onViewerHostReady,
		isKernelScreenVisible,
		onKernelScreenReady,
		onKernelScreenError
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

	{#if progress}
		<div class="viewer-status-row">
			<div class="progress-pill viewer-progress-pill">
				<span class="label">Carregamento</span>
				<strong>{progress.percentage.toFixed(0)}%</strong>
				<span>{progress.stage}{progress.subStage ? ` / ${progress.subStage}` : ''}</span>
			</div>
		</div>
	{/if}

	<ViewerDropzone
		{currentDocument}
		{isDragActive}
		{isTauriRuntime}
		{onDragEnter}
		{onDragOver}
		{onDragLeave}
		{onDrop}
		onHostReady={onViewerHostReady}
		{isKernelScreenVisible}
		isDark={backgroundTheme === 'dark'}
		{onKernelScreenReady}
		{onKernelScreenError}
	/>
</section>
