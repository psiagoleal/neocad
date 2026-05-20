<script lang="ts">
	import { onMount } from 'svelte';
	import type { CadViewerDocumentState } from '$lib/types/cad';

	type ViewerDropzoneProps = {
		currentDocument: CadViewerDocumentState | null;
		isDragActive: boolean;
		isTauriRuntime: boolean;
		onDragEnter: (event: DragEvent) => void;
		onDragOver: (event: DragEvent) => void;
		onDragLeave: (event: DragEvent) => void;
		onDrop: (event: DragEvent) => void | Promise<void>;
		onHostReady: (element: HTMLDivElement) => void;
	};

	let {
		currentDocument,
		isDragActive,
		isTauriRuntime,
		onDragEnter,
		onDragOver,
		onDragLeave,
		onDrop,
		onHostReady
	}: ViewerDropzoneProps = $props();

	let viewerElement: HTMLDivElement | null = $state(null);

	onMount(() => {
		if (viewerElement != null) {
			onHostReady(viewerElement);
		}
	});
</script>

<div
	class:drag-active={isDragActive}
	class="viewer-surface"
	role="region"
	aria-label="Área de visualização CAD com suporte a arrastar e soltar"
	ondragenter={onDragEnter}
	ondragover={onDragOver}
	ondragleave={onDragLeave}
	ondrop={onDrop}
>
	<div class="viewer-container" bind:this={viewerElement}></div>

	{#if !currentDocument}
		<div class="viewer-overlay">
			<h3>NeoCAD Viewer</h3>
			<p>
				Abra um arquivo local para iniciar a visualização.
				{#if isTauriRuntime}
					Em runtime Tauri, o fluxo usa diálogo nativo e leitura segura do sistema de arquivos.
				{:else}
					Em navegador, a abertura funciona por seleção local ou arrastar e soltar.
				{/if}
			</p>
			<p class="drop-hint">Você também pode arrastar um arquivo DWG ou DXF para esta área.</p>
		</div>
	{/if}

	{#if isDragActive}
		<div class="drop-overlay">
			<div>
				<strong>Solte o arquivo CAD aqui</strong>
				<span>Compatível com DWG e DXF</span>
			</div>
		</div>
	{/if}
</div>
