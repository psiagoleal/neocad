<!-- Caminho relativo: src/routes/+page.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { appMetadata, nextMilestones, primaryStack, supportedTargets } from '$lib/config/app';
	import {
		createCadDocumentPayloadFromFile,
		getCadRuntimeLabel,
		readCadDocumentFromPath,
		selectCadDocument
	} from '$lib/services/cad-file';
	import {
		clearRecentDocuments,
		listRecentDocuments,
		registerRecentDocument
	} from '$lib/services/recent-documents';
	import type {
		CadDocumentPayload,
		CadRecentDocument,
		CadViewerDocumentState,
		CadViewerMessage,
		CadViewerProgressState
	} from '$lib/types/cad';
	import { NeoCadViewer } from '$lib/viewer/neocad-viewer';

	const runtimeLabel = getCadRuntimeLabel();
	const isTauriRuntime = runtimeLabel === 'Tauri';
	const commandSuggestions = [
		'open',
		'zoom',
		'pan',
		'select',
		'erase',
		'move',
		'copy',
		'rotate'
	] as const;

	let viewerHost: HTMLDivElement | null = $state(null);
	let viewerController: NeoCadViewer | null = $state(null);
	let currentDocument: CadViewerDocumentState | null = $state(null);
	let recentDocuments: CadRecentDocument[] = $state([]);
	let progress: CadViewerProgressState | null = $state(null);
	let notifications: CadViewerMessage[] = $state([]);
	let backgroundTheme: 'light' | 'dark' = $state('dark');
	let commandInput = $state('zoom');
	let isOpening = $state(false);
	let isViewerReady = $state(false);
	let isDragActive = $state(false);
	let notificationSequence = 0;

	function pushNotification(kind: CadViewerMessage['kind'], text: string): void {
		notifications = [
			{
				id: `notification-${++notificationSequence}`,
				kind,
				text
			},
			...notifications
		].slice(0, 5);
	}

	function refreshRecentDocuments(): void {
		recentDocuments = listRecentDocuments();
	}

	function rememberDocument(state: CadViewerDocumentState): void {
		recentDocuments = registerRecentDocument({
			fileName: state.fileName,
			path: state.path,
			source: state.source ?? 'browser',
			openedAt: new Date().toISOString()
		});
	}

	function clearProgress(): void {
		progress = null;
	}

	function preventFileDropDefaults(event: DragEvent): void {
		event.preventDefault();
		event.stopPropagation();
	}

	async function openCadPayload(payload: CadDocumentPayload): Promise<void> {
		if (viewerController == null) {
			throw new Error('Viewer CAD ainda não foi inicializado.');
		}

		const isSuccess = await viewerController.openDocument(payload, 'write');

		if (!isSuccess) {
			pushNotification('error', `Falha ao carregar ${payload.fileName}.`);
		}
	}

	async function openCadDrawing(): Promise<void> {
		if (viewerController == null || isOpening) {
			return;
		}

		isOpening = true;
		progress = {
			percentage: 0,
			stage: 'AWAITING_SELECTION'
		};

		try {
			const payload = await selectCadDocument();

			if (payload == null) {
				pushNotification('info', 'A abertura do desenho foi cancelada.');
				clearProgress();
				return;
			}

			await openCadPayload(payload);
		} catch (error) {
			pushNotification(
				'error',
				error instanceof Error ? error.message : 'Falha inesperada ao abrir o desenho CAD.'
			);
		} finally {
			isOpening = false;
		}
	}

	async function openRecentDrawing(recentDocument: CadRecentDocument): Promise<void> {
		if (!isTauriRuntime) {
			pushNotification(
				'info',
				'Reabertura de recentes por caminho completo está disponível apenas no runtime Tauri.'
			);
			return;
		}

		if (recentDocument.path == null) {
			pushNotification(
				'warning',
				'Este item recente não possui caminho persistido para reabertura.'
			);
			return;
		}

		try {
			isOpening = true;
			progress = {
				percentage: 0,
				stage: 'OPENING_RECENT'
			};

			const payload = await readCadDocumentFromPath(recentDocument.path);
			await openCadPayload(payload);
		} catch (error) {
			pushNotification(
				'error',
				error instanceof Error ? error.message : 'Falha ao reabrir o documento recente selecionado.'
			);
		} finally {
			isOpening = false;
		}
	}

	async function handleFileDrop(event: DragEvent): Promise<void> {
		preventFileDropDefaults(event);
		isDragActive = false;

		if (viewerController == null || isOpening) {
			return;
		}

		const file = event.dataTransfer?.files?.[0];

		if (file == null) {
			pushNotification('warning', 'Nenhum arquivo foi detectado no drop.');
			return;
		}

		try {
			isOpening = true;
			progress = {
				percentage: 0,
				stage: 'READING_DROPPED_FILE'
			};

			const source = isTauriRuntime ? 'tauri' : 'browser';
			const payload = await createCadDocumentPayloadFromFile(file, source);
			await openCadPayload(payload);
			pushNotification('info', `Arquivo recebido por arrastar e soltar: ${payload.fileName}`);
		} catch (error) {
			pushNotification(
				'error',
				error instanceof Error
					? error.message
					: 'Falha inesperada ao processar o arquivo arrastado.'
			);
		} finally {
			isOpening = false;
		}
	}

	function fitDrawingToView(): void {
		viewerController?.zoomToFit();
		pushNotification('info', 'Ajuste de visualização aplicado ao desenho.');
	}

	function toggleViewerBackground(): void {
		backgroundTheme = viewerController?.toggleBackground() ?? backgroundTheme;
		pushNotification(
			'info',
			`Tema do canvas alterado para ${backgroundTheme === 'dark' ? 'escuro' : 'claro'}.`
		);
	}

	function executeCadCommand(): void {
		const normalizedCommand = commandInput.trim().toLowerCase();

		if (viewerController == null || normalizedCommand.length === 0) {
			return;
		}

		try {
			viewerController.executeCommand(normalizedCommand);
			pushNotification('info', `Comando enviado ao viewer: ${normalizedCommand}`);
		} catch (error) {
			pushNotification(
				'error',
				error instanceof Error ? error.message : 'Não foi possível executar o comando CAD.'
			);
		}
	}

	onMount(() => {
		refreshRecentDocuments();

		if (viewerHost == null) {
			pushNotification('error', 'O container principal do viewer não foi inicializado.');
			return;
		}

		const controller = new NeoCadViewer({
			onOpenRequested: openCadDrawing,
			onProgress: (value) => {
				progress = value;
			},
			onMessage: ({ kind, text }) => {
				pushNotification(kind, text);
			},
			onDocumentActivated: (state) => {
				currentDocument = state;
				clearProgress();
				rememberDocument(state);
				pushNotification('success', `Desenho carregado com sucesso: ${state.docTitle}`);
			}
		});

		viewerController = controller;

		void controller
			.mount(viewerHost)
			.then(() => {
				isViewerReady = true;
				pushNotification('success', 'Viewer CAD inicializado e pronto para abrir desenhos.');
			})
			.catch((error) => {
				pushNotification(
					'error',
					error instanceof Error
						? error.message
						: 'Falha ao inicializar a integração com o viewer CAD.'
				);
			});

		return () => {
			isViewerReady = false;
			currentDocument = null;
			progress = null;
			void controller.destroy();
		};
	});
</script>

<svelte:head>
	<title>{appMetadata.name} — Fase 2</title>
	<meta
		name="description"
		content="Integração inicial do NeoCAD com o core framework-agnostic do ecossistema cad-viewer para abrir arquivos DWG e DXF localmente."
	/>
</svelte:head>

<section class="workspace-shell">
	<aside class="sidebar card-panel">
		<div class="sidebar-block">
			<p class="eyebrow">Integração do viewer</p>
			<h1>{appMetadata.name}</h1>
			<p class="tagline">{appMetadata.tagline}</p>
			<p class="support-copy">
				Nesta etapa, o NeoCAD passa a usar o pacote framework-agnostic
				<code>@mlightcad/cad-simple-viewer</code>, que é a base mais adequada para integrar o
				ecosistema do upstream <code>cad-viewer</code> em uma interface Svelte.
			</p>
		</div>

		<div class="sidebar-block meta-grid">
			<div>
				<span class="label">Status</span>
				<strong>{appMetadata.status}</strong>
			</div>
			<div>
				<span class="label">Runtime</span>
				<strong>{runtimeLabel}</strong>
			</div>
			<div>
				<span class="label">Canvas</span>
				<strong>{backgroundTheme === 'dark' ? 'Escuro' : 'Claro'}</strong>
			</div>
			<div>
				<span class="label">Viewer</span>
				<strong>{isViewerReady ? 'Pronto' : 'Inicializando'}</strong>
			</div>
		</div>

		<div class="sidebar-block action-row">
			<button
				class="primary-button"
				type="button"
				onclick={openCadDrawing}
				disabled={!isViewerReady || isOpening}
			>
				{isOpening ? 'Abrindo desenho...' : 'Abrir desenho CAD'}
			</button>
			<button
				class="secondary-button"
				type="button"
				onclick={fitDrawingToView}
				disabled={!currentDocument}
			>
				Ajustar à vista
			</button>
			<button
				class="secondary-button"
				type="button"
				onclick={toggleViewerBackground}
				disabled={!isViewerReady}
			>
				Alternar fundo
			</button>
		</div>

		<div class="sidebar-block">
			<h2>Comando CAD</h2>
			<div class="command-box">
				<input
					type="text"
					bind:value={commandInput}
					placeholder="Ex.: zoom, line, erase, move"
					onkeydown={(event) => {
						if (event.key === 'Enter') {
							executeCadCommand();
						}
					}}
				/>
				<button type="button" onclick={executeCadCommand} disabled={!isViewerReady}>Executar</button
				>
			</div>

			<div class="chips">
				{#each commandSuggestions as suggestion (suggestion)}
					<button class="chip" type="button" onclick={() => (commandInput = suggestion)}
						>{suggestion}</button
					>
				{/each}
			</div>
		</div>

		<div class="sidebar-block">
			<h2>Documento ativo</h2>
			{#if currentDocument}
				<ul class="details-list">
					<li><span>Título</span><strong>{currentDocument.docTitle}</strong></li>
					<li><span>Arquivo</span><strong>{currentDocument.fileName}</strong></li>
					<li><span>Modo</span><strong>{currentDocument.mode}</strong></li>
					<li><span>Origem</span><strong>{currentDocument.source ?? 'n/d'}</strong></li>
				</ul>
			{:else}
				<p class="empty-copy">
					Nenhum desenho aberto ainda. Use o botão acima para carregar um arquivo DWG ou DXF.
				</p>
			{/if}
		</div>

		<div class="sidebar-block">
			<div class="section-header">
				<h2>Recentes</h2>
				<button
					class="inline-action"
					type="button"
					onclick={() => {
						clearRecentDocuments();
						recentDocuments = [];
					}}
				>
					Limpar
				</button>
			</div>

			{#if recentDocuments.length > 0}
				<ul class="recent-list">
					{#each recentDocuments as recentDocument (recentDocument.openedAt + recentDocument.fileName)}
						<li>
							<div>
								<strong>{recentDocument.fileName}</strong>
								<span>{recentDocument.path ?? 'origem local sem caminho persistido'}</span>
							</div>
							<button
								type="button"
								onclick={() => openRecentDrawing(recentDocument)}
								disabled={!isTauriRuntime || recentDocument.path == null || isOpening}
							>
								Abrir
							</button>
						</li>
					{/each}
				</ul>
			{:else}
				<p class="empty-copy">Os desenhos abertos recentemente aparecerão aqui.</p>
			{/if}
		</div>

		<div class="sidebar-block">
			<h2>Stack e próximos passos</h2>
			<ul class="details-list compact-list">
				{#each primaryStack as item (item)}
					<li><span>Stack</span><strong>{item}</strong></li>
				{/each}
			</ul>
			<ul class="plain-list">
				{#each nextMilestones as milestone (milestone)}
					<li>{milestone}</li>
				{/each}
			</ul>
		</div>

		<div class="sidebar-block">
			<h2>Plataformas alvo</h2>
			<div class="chips">
				{#each supportedTargets as target (target)}
					<span class="status-chip">{target}</span>
				{/each}
			</div>
		</div>
	</aside>

	<div class="viewer-layout">
		<section class="viewer-frame card-panel">
			<header class="viewer-header">
				<div>
					<p class="eyebrow">Canvas CAD</p>
					<h2>Área de visualização</h2>
				</div>
				{#if progress}
					<div class="progress-pill">
						<strong>{progress.percentage.toFixed(0)}%</strong>
						<span>{progress.stage}{progress.subStage ? ` / ${progress.subStage}` : ''}</span>
					</div>
				{/if}
			</header>

			<div
				class:drag-active={isDragActive}
				class="viewer-surface"
				role="region"
				aria-label="Área de visualização CAD com suporte a arrastar e soltar"
				ondragenter={(event) => {
					preventFileDropDefaults(event);
					isDragActive = true;
				}}
				ondragover={preventFileDropDefaults}
				ondragleave={(event) => {
					preventFileDropDefaults(event);
					if (event.currentTarget === event.target) {
						isDragActive = false;
					}
				}}
				ondrop={handleFileDrop}
			>
				<div class="viewer-container" bind:this={viewerHost}></div>

				{#if !currentDocument}
					<div class="viewer-overlay">
						<h3>NeoCAD Viewer</h3>
						<p>
							Abra um arquivo local para iniciar a visualização. Em runtime Tauri, o fluxo usa
							diálogo nativo e leitura segura do sistema de arquivos.
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
		</section>

		<section class="notifications-panel card-panel">
			<header class="viewer-header">
				<div>
					<p class="eyebrow">Mensagens</p>
					<h2>Status da integração</h2>
				</div>
			</header>

			{#if notifications.length > 0}
				<ul class="notifications-list">
					{#each notifications as notification (notification.id)}
						<li class={`notification ${notification.kind}`}>
							<strong>{notification.kind}</strong>
							<span>{notification.text}</span>
						</li>
					{/each}
				</ul>
			{:else}
				<p class="empty-copy">As mensagens do viewer e do fluxo de abertura aparecerão aqui.</p>
			{/if}
		</section>
	</div>
</section>

<style>
	:global(body) {
		margin: 0;
		min-height: 100vh;
		font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
		background:
			radial-gradient(circle at top, rgba(77, 134, 255, 0.16), transparent 22%),
			linear-gradient(180deg, #07111f 0%, #091728 52%, #0d1f35 100%);
		color: #edf4ff;
	}

	:global(button),
	:global(input) {
		font: inherit;
	}

	.workspace-shell {
		display: grid;
		grid-template-columns: minmax(320px, 420px) minmax(0, 1fr);
		gap: 1rem;
		max-width: 1680px;
		margin: 0 auto;
		padding: 1rem;
	}

	.viewer-layout {
		display: grid;
		gap: 1rem;
		min-width: 0;
	}

	.card-panel {
		border-radius: 1.1rem;
		background: rgba(8, 20, 37, 0.82);
		border: 1px solid rgba(127, 179, 255, 0.14);
		backdrop-filter: blur(12px);
		box-shadow: 0 18px 44px rgba(0, 0, 0, 0.2);
	}

	.sidebar {
		display: grid;
		align-content: start;
		gap: 1rem;
		padding: 1rem;
	}

	.sidebar-block {
		display: grid;
		gap: 0.85rem;
	}

	.meta-grid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}

	.meta-grid > div {
		padding: 0.85rem;
		border-radius: 0.85rem;
		background: rgba(255, 255, 255, 0.04);
	}

	.label {
		display: block;
		font-size: 0.76rem;
		text-transform: uppercase;
		letter-spacing: 0.12em;
		color: #8fb7ff;
		margin-bottom: 0.35rem;
	}

	.eyebrow {
		margin: 0;
		text-transform: uppercase;
		letter-spacing: 0.14em;
		font-size: 0.78rem;
		font-weight: 700;
		color: #7fb3ff;
	}

	h1,
	h2,
	h3,
	p {
		margin: 0;
	}

	h1 {
		font-size: clamp(2.4rem, 6vw, 3.8rem);
		line-height: 0.95;
	}

	h2 {
		font-size: 1.1rem;
	}

	.tagline,
	.support-copy,
	.empty-copy,
	.drop-hint {
		line-height: 1.7;
		color: #c8daf8;
	}

	.action-row,
	.command-box,
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 0.65rem;
	}

	button,
	.chip,
	.status-chip {
		border: none;
		border-radius: 999px;
		padding: 0.8rem 1rem;
		font-weight: 700;
	}

	button {
		cursor: pointer;
	}

	button:disabled {
		cursor: not-allowed;
		opacity: 0.6;
	}

	.primary-button {
		background: linear-gradient(135deg, #87b8ff 0%, #4f8fff 100%);
		color: #07111f;
	}

	.secondary-button,
	.chip,
	.inline-action,
	.recent-list button {
		background: rgba(127, 179, 255, 0.1);
		color: #e8f1ff;
		border: 1px solid rgba(127, 179, 255, 0.16);
	}

	.inline-action {
		padding: 0.5rem 0.8rem;
	}

	.status-chip {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		background: rgba(127, 179, 255, 0.14);
		color: #e8f1ff;
	}

	.command-box {
		align-items: center;
	}

	.command-box input {
		flex: 1 1 220px;
		padding: 0.85rem 1rem;
		border-radius: 0.85rem;
		border: 1px solid rgba(127, 179, 255, 0.16);
		background: rgba(3, 10, 19, 0.72);
		color: #edf4ff;
	}

	.command-box button {
		background: #2b63c7;
		color: #f2f7ff;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
	}

	.details-list,
	.plain-list,
	.notifications-list,
	.recent-list {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.details-list {
		display: grid;
		gap: 0.7rem;
	}

	.details-list li,
	.recent-list li {
		display: flex;
		justify-content: space-between;
		gap: 1rem;
		padding: 0.75rem 0.9rem;
		border-radius: 0.8rem;
		background: rgba(255, 255, 255, 0.04);
	}

	.details-list span {
		color: #9ab8e8;
	}

	.compact-list li {
		align-items: center;
	}

	.recent-list {
		display: grid;
		gap: 0.7rem;
	}

	.recent-list div {
		display: grid;
		gap: 0.25rem;
		min-width: 0;
	}

	.recent-list span {
		font-size: 0.84rem;
		color: #9ab8e8;
		word-break: break-word;
	}

	.plain-list {
		display: grid;
		gap: 0.6rem;
		padding-left: 1rem;
		list-style: disc;
		color: #dce9ff;
	}

	.viewer-frame,
	.notifications-panel {
		padding: 1rem;
	}

	.viewer-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
		margin-bottom: 0.9rem;
	}

	.progress-pill {
		display: grid;
		gap: 0.1rem;
		padding: 0.7rem 0.85rem;
		border-radius: 0.8rem;
		background: rgba(127, 179, 255, 0.12);
		color: #eaf2ff;
	}

	.progress-pill span {
		font-size: 0.85rem;
		color: #bdd2f8;
	}

	.viewer-surface {
		position: relative;
		min-height: 72vh;
		border-radius: 0.95rem;
		overflow: hidden;
		background: #081121;
		border: 1px solid rgba(127, 179, 255, 0.12);
	}

	.viewer-surface.drag-active {
		outline: 2px dashed rgba(135, 184, 255, 0.8);
		outline-offset: -0.35rem;
	}

	.viewer-container {
		position: absolute;
		inset: 0;
	}

	.viewer-overlay,
	.drop-overlay {
		position: absolute;
		inset: 0;
		display: grid;
		place-content: center;
		gap: 0.75rem;
		padding: 2rem;
		text-align: center;
		color: #edf4ff;
	}

	.viewer-overlay {
		background:
			radial-gradient(circle at center, rgba(127, 179, 255, 0.16), transparent 34%),
			rgba(8, 17, 33, 0.78);
		pointer-events: none;
	}

	.viewer-overlay p {
		max-width: 34rem;
		line-height: 1.7;
		color: #cfddf8;
	}

	.drop-overlay {
		background: rgba(7, 17, 31, 0.76);
		backdrop-filter: blur(8px);
	}

	.drop-overlay div {
		display: grid;
		gap: 0.35rem;
		padding: 1.3rem 1.5rem;
		border-radius: 1rem;
		background: rgba(127, 179, 255, 0.14);
		border: 1px solid rgba(127, 179, 255, 0.22);
	}

	.notifications-list {
		display: grid;
		gap: 0.7rem;
	}

	.notification {
		display: grid;
		gap: 0.3rem;
		padding: 0.85rem 1rem;
		border-radius: 0.85rem;
		border: 1px solid transparent;
	}

	.notification strong {
		text-transform: uppercase;
		letter-spacing: 0.08em;
		font-size: 0.76rem;
	}

	.notification.info {
		background: rgba(127, 179, 255, 0.12);
		border-color: rgba(127, 179, 255, 0.18);
	}

	.notification.success {
		background: rgba(49, 196, 141, 0.12);
		border-color: rgba(49, 196, 141, 0.24);
	}

	.notification.warning {
		background: rgba(255, 189, 89, 0.13);
		border-color: rgba(255, 189, 89, 0.22);
	}

	.notification.error {
		background: rgba(255, 107, 107, 0.13);
		border-color: rgba(255, 107, 107, 0.22);
	}

	code {
		padding: 0.15rem 0.4rem;
		border-radius: 0.4rem;
		background: rgba(255, 255, 255, 0.09);
		font-size: 0.95em;
	}

	@media (max-width: 1180px) {
		.workspace-shell {
			grid-template-columns: 1fr;
		}

		.viewer-surface {
			min-height: 56vh;
		}
	}
</style>
