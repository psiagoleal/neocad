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
		clearRecentDocuments as clearStoredRecentDocuments,
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

	type WorkspaceView = 'home' | 'viewer' | 'about';

	const runtimeLabel = getCadRuntimeLabel();
	const isTauriRuntime = runtimeLabel === 'Tauri';

	let viewerHost: HTMLDivElement | null = $state(null);
	let viewerController: NeoCadViewer | null = $state(null);
	let activeWorkspace: WorkspaceView = $state('home');
	let hasVisitedViewerWorkspace = $state(false);
	let currentDocument: CadViewerDocumentState | null = $state(null);
	let recentDocuments: CadRecentDocument[] = $state([]);
	let progress: CadViewerProgressState | null = $state(null);
	let notifications: CadViewerMessage[] = $state([]);
	let backgroundTheme: 'light' | 'dark' = $state('dark');
	let isOpening = $state(false);
	let isViewerReady = $state(false);
	let isDragActive = $state(false);
	let isMessagesVisible = $state(false);
	let unreadMessages = $state(0);
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

		if (!isMessagesVisible) {
			unreadMessages += 1;
		}
	}

	function openMessagesDock(): void {
		isMessagesVisible = true;
		unreadMessages = 0;
	}

	function closeMessagesDock(): void {
		isMessagesVisible = false;
	}

	function toggleMessagesDock(): void {
		if (isMessagesVisible) {
			closeMessagesDock();
			return;
		}

		openMessagesDock();
	}

	function showHomeWorkspace(): void {
		activeWorkspace = 'home';
	}

	function showAboutWorkspace(): void {
		activeWorkspace = 'about';
	}

	function showViewerWorkspace(force = false): void {
		if (force || hasVisitedViewerWorkspace || currentDocument != null) {
			activeWorkspace = 'viewer';
			return;
		}

		pushNotification('info', 'Abra um desenho CAD para entrar no workspace principal do canvas.');
	}

	async function refreshRecentDocuments(): Promise<void> {
		try {
			recentDocuments = await listRecentDocuments();
		} catch (error) {
			recentDocuments = [];
			pushNotification(
				'warning',
				error instanceof Error
					? error.message
					: 'Não foi possível carregar a lista de desenhos recentes.'
			);
		}
	}

	async function rememberDocument(state: CadViewerDocumentState): Promise<void> {
		try {
			recentDocuments = await registerRecentDocument({
				fileName: state.fileName,
				path: state.path,
				source: state.source ?? 'browser',
				openedAt: new Date().toISOString()
			});
		} catch (error) {
			pushNotification(
				'warning',
				error instanceof Error
					? error.message
					: 'Não foi possível persistir o histórico de desenhos recentes.'
			);
		}
	}

	async function clearRecentDocumentsList(): Promise<void> {
		try {
			await clearStoredRecentDocuments();
			recentDocuments = [];
			pushNotification('info', 'Lista de desenhos recentes limpa.');
		} catch (error) {
			pushNotification(
				'warning',
				error instanceof Error
					? error.message
					: 'Não foi possível limpar a lista de desenhos recentes.'
			);
		}
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

			await openCadPayload(await readCadDocumentFromPath(recentDocument.path));
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

	onMount(() => {
		void refreshRecentDocuments();

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
				hasVisitedViewerWorkspace = true;
				activeWorkspace = 'viewer';
				clearProgress();
				void rememberDocument(state);
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
	<title>{appMetadata.name} — Workspace desktop</title>
	<meta
		name="description"
		content="NeoCAD em fluxo desktop com tela inicial compacta, canvas principal priorizado e informações complementares em uma tela dedicada."
	/>
</svelte:head>

<section class="app-shell">
	<header class="topbar card-panel">
		<div class="topbar-brand">
			<div>
				<p class="eyebrow">NeoCAD Workspace</p>
				<h1>{appMetadata.name}</h1>
			</div>
			<div class="topbar-status">
				<span class="status-chip">{appMetadata.status}</span>
				<span class="status-chip">Runtime {runtimeLabel}</span>
			</div>
		</div>

		<nav class="topbar-nav" aria-label="Navegação principal do workspace">
			<button
				class:active={activeWorkspace === 'home'}
				class="nav-button"
				type="button"
				onclick={showHomeWorkspace}
			>
				Início
			</button>
			<button
				class:active={activeWorkspace === 'viewer'}
				class="nav-button"
				type="button"
				onclick={() => showViewerWorkspace()}
				disabled={!hasVisitedViewerWorkspace && currentDocument == null}
			>
				Canvas CAD
			</button>
			<button
				class:active={activeWorkspace === 'about'}
				class="nav-button"
				type="button"
				onclick={showAboutWorkspace}
			>
				Sobre
			</button>
		</nav>

		<div class="topbar-actions">
			<button class="utility-button" type="button" onclick={toggleMessagesDock}>
				Mensagens
				{#if unreadMessages > 0}
					<span class="badge">{unreadMessages}</span>
				{/if}
			</button>

			<div class="document-pill">
				<span class="label">Documento</span>
				<strong>{currentDocument?.fileName ?? 'Nenhum desenho aberto'}</strong>
			</div>

			<button
				class="primary-button"
				type="button"
				onclick={openCadDrawing}
				disabled={!isViewerReady || isOpening}
			>
				{isOpening ? 'Abrindo...' : 'Abrir desenho'}
			</button>
		</div>
	</header>

	<div class="workspace-stage">
		<section
			class:workspace-active={activeWorkspace === 'home'}
			class="workspace-screen home-screen"
			aria-hidden={activeWorkspace !== 'home'}
		>
			<div class="home-grid">
				<section class="home-hero card-panel">
					<p class="eyebrow">Fluxo principal</p>
					<h2>O viewer agora é o foco da aplicação.</h2>
					<p class="support-copy compact-copy">
						Abra um desenho para entrar direto no canvas. Informações de planejamento e detalhes de
						implementação ficam disponíveis separadamente em <strong>Sobre</strong>.
					</p>

					<div class="hero-actions">
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
							onclick={() => showViewerWorkspace(true)}
						>
							Ir para o canvas
						</button>
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
						<li>O campo de comando redundante foi removido para manter o foco no viewer.</li>
						<li>O dock de mensagens permanece opcional e pode ficar recolhido durante o uso.</li>
					</ul>
				</section>

				<section class="home-panel card-panel">
					<div class="section-header">
						<h2>Desenhos recentes</h2>
						<button class="inline-action" type="button" onclick={clearRecentDocumentsList}>
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
				</section>
			</div>
		</section>

		<section
			class:workspace-active={activeWorkspace === 'viewer'}
			class="workspace-screen viewer-screen"
			aria-hidden={activeWorkspace !== 'viewer'}
		>
			<section class="viewer-frame card-panel viewer-focus-frame">
				<header class="viewer-header compact-header">
					<div>
						<p class="eyebrow">Canvas CAD</p>
						<h2>{currentDocument?.docTitle ?? 'Área principal de visualização'}</h2>
					</div>

					<div class="viewer-toolbar">
						<button
							class="secondary-button"
							type="button"
							onclick={fitDrawingToView}
							disabled={!currentDocument}
						>
							Ajustar
						</button>
						<button
							class="secondary-button"
							type="button"
							onclick={toggleViewerBackground}
							disabled={!isViewerReady}
						>
							Fundo {backgroundTheme === 'dark' ? 'escuro' : 'claro'}
						</button>
						<button
							class="secondary-button"
							type="button"
							onclick={openCadDrawing}
							disabled={!isViewerReady || isOpening}
						>
							{isOpening ? 'Abrindo...' : 'Abrir'}
						</button>
					</div>
				</header>

				<div class="viewer-meta-strip">
					<div class="meta-pill">
						<span class="label">Arquivo</span>
						<strong>{currentDocument?.fileName ?? 'Nenhum desenho aberto'}</strong>
					</div>
					<div class="meta-pill">
						<span class="label">Modo</span>
						<strong>{currentDocument?.mode ?? 'Aguardando'}</strong>
					</div>
					<div class="meta-pill">
						<span class="label">Origem</span>
						<strong>{currentDocument?.source ?? runtimeLabel}</strong>
					</div>
					{#if progress}
						<div class="progress-pill">
							<span class="label">Carregamento</span>
							<strong>{progress.percentage.toFixed(0)}%</strong>
							<span>{progress.stage}{progress.subStage ? ` / ${progress.subStage}` : ''}</span>
						</div>
					{/if}
				</div>

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
							<p class="drop-hint">
								Você também pode arrastar um arquivo DWG ou DXF para esta área.
							</p>
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
		</section>

		<section
			class:workspace-active={activeWorkspace === 'about'}
			class="workspace-screen about-screen"
			aria-hidden={activeWorkspace !== 'about'}
		>
			<div class="about-grid">
				<section class="home-panel card-panel">
					<p class="eyebrow">Sobre o app</p>
					<h2>Detalhes de planejamento e implementação</h2>
					<p class="support-copy compact-copy">
						Esta área concentra informações institucionais e técnicas para que a tela principal
						fique mais limpa e focada no trabalho com o desenho.
					</p>
					<ul class="details-list">
						<li><span>Status</span><strong>{appMetadata.status}</strong></li>
						<li><span>Runtime atual</span><strong>{runtimeLabel}</strong></li>
						<li><span>Licença</span><strong>{appMetadata.license}</strong></li>
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
		</section>

		<section class:is-visible={isMessagesVisible} class="messages-dock card-panel">
			<header class="messages-header compact-header">
				<div>
					<p class="eyebrow">Mensagens</p>
					<h2>Status da integração</h2>
				</div>
				<button class="inline-action" type="button" onclick={closeMessagesDock}>Ocultar</button>
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

		{#if !isMessagesVisible}
			<button class="messages-fab" type="button" onclick={openMessagesDock}>
				Mensagens
				{#if unreadMessages > 0}
					<span class="badge">{unreadMessages}</span>
				{/if}
			</button>
		{/if}
	</div>
</section>

<style>
	:global(html),
	:global(body) {
		height: 100%;
		overflow: hidden;
	}

	:global(body) {
		margin: 0;
		min-height: 100dvh;
		font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
		background:
			radial-gradient(circle at top, rgba(77, 134, 255, 0.12), transparent 24%),
			linear-gradient(180deg, #07111f 0%, #091728 52%, #0d1f35 100%);
		color: #edf4ff;
	}

	:global(*),
	:global(*::before),
	:global(*::after) {
		box-sizing: border-box;
	}

	:global(button),
	:global(input) {
		font: inherit;
	}

	.app-shell {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		gap: 0.75rem;
		height: 100dvh;
		width: 100%;
		max-width: 100%;
		margin: 0 auto;
		padding: 0.75rem;
		overflow: hidden;
	}

	.card-panel {
		border-radius: 1rem;
		background: rgba(8, 20, 37, 0.84);
		border: 1px solid rgba(127, 179, 255, 0.12);
		backdrop-filter: blur(10px);
		box-shadow: 0 14px 32px rgba(0, 0, 0, 0.18);
	}

	.topbar {
		display: grid;
		grid-template-columns: auto 1fr auto;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem 1rem;
		min-height: 0;
	}

	.topbar-brand,
	.topbar-status,
	.topbar-actions,
	.topbar-nav,
	.hero-actions,
	.chips,
	.viewer-toolbar {
		display: flex;
		align-items: center;
		gap: 0.55rem;
		flex-wrap: wrap;
	}

	.topbar-brand {
		gap: 0.75rem;
	}

	.topbar-status {
		row-gap: 0.35rem;
	}

	.topbar-nav {
		justify-content: center;
	}

	.topbar-actions {
		justify-content: flex-end;
		min-width: 0;
	}

	.workspace-stage {
		position: relative;
		min-height: 0;
		height: 100%;
		border-radius: 1.15rem;
		overflow: hidden;
		background: rgba(4, 11, 20, 0.34);
		border: 1px solid rgba(127, 179, 255, 0.12);
	}

	.workspace-screen {
		position: absolute;
		inset: 0;
		opacity: 0;
		visibility: hidden;
		pointer-events: none;
		transform: translateY(0.5rem);
		transition:
			opacity 180ms ease,
			transform 180ms ease,
			visibility 180ms ease;
	}

	.workspace-active {
		opacity: 1;
		visibility: visible;
		pointer-events: auto;
		transform: translateY(0);
	}

	.home-screen,
	.about-screen {
		padding: 0.85rem;
		overflow: auto;
	}

	.home-grid,
	.about-grid {
		display: grid;
		gap: 0.85rem;
	}

	.home-grid {
		grid-template-columns: minmax(0, 1.2fr) minmax(320px, 0.9fr);
		align-items: start;
	}

	.about-grid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
		align-items: start;
	}

	.home-hero,
	.home-panel,
	.viewer-frame,
	.messages-dock {
		padding: 0.9rem;
	}

	.home-hero,
	.home-panel {
		display: grid;
		gap: 0.8rem;
		align-content: start;
	}

	.viewer-screen {
		display: grid;
		padding: 0.85rem;
		overflow: hidden;
	}

	.viewer-focus-frame {
		display: grid;
		grid-template-rows: auto auto minmax(0, 1fr);
		min-width: 0;
		min-height: 0;
	}

	.compact-header,
	.section-header,
	.messages-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.75rem;
	}

	.compact-header {
		margin-bottom: 0.75rem;
	}

	.meta-grid {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 0.6rem;
	}

	.meta-grid > div,
	.meta-pill,
	.document-pill,
	.progress-pill {
		padding: 0.7rem 0.8rem;
		border-radius: 0.8rem;
		background: rgba(255, 255, 255, 0.04);
		min-width: 0;
	}

	.viewer-meta-strip {
		display: grid;
		grid-template-columns: repeat(4, minmax(0, 1fr));
		gap: 0.6rem;
		margin-bottom: 0.75rem;
	}

	.document-pill {
		display: grid;
		gap: 0.15rem;
		min-width: min(18rem, 100%);
	}

	.meta-pill,
	.progress-pill {
		display: grid;
		gap: 0.2rem;
	}

	.progress-pill span:last-child {
		font-size: 0.8rem;
		color: #a8c4ef;
	}

	.label {
		display: block;
		font-size: 0.68rem;
		text-transform: uppercase;
		letter-spacing: 0.12em;
		color: #8fb7ff;
		margin-bottom: 0.2rem;
	}

	.eyebrow {
		margin: 0;
		text-transform: uppercase;
		letter-spacing: 0.13em;
		font-size: 0.68rem;
		font-weight: 700;
		color: #7fb3ff;
	}

	h1,
	h2,
	h3,
	p,
	strong,
	span {
		margin: 0;
	}

	h1 {
		font-size: 1.3rem;
	}

	h2 {
		font-size: 1rem;
	}

	h3 {
		font-size: 0.96rem;
	}

	.support-copy,
	.empty-copy,
	.drop-hint {
		line-height: 1.55;
		color: #c8daf8;
	}

	.compact-copy {
		font-size: 0.92rem;
	}

	button,
	.status-chip,
	.nav-button,
	.messages-fab,
	.utility-button {
		border: none;
		border-radius: 999px;
		padding: 0.62rem 0.88rem;
		font-weight: 700;
		font-size: 0.9rem;
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
	.inline-action,
	.recent-list button,
	.nav-button,
	.messages-fab,
	.utility-button {
		background: rgba(127, 179, 255, 0.1);
		color: #e8f1ff;
		border: 1px solid rgba(127, 179, 255, 0.16);
	}

	.nav-button.active {
		background: rgba(127, 179, 255, 0.2);
		border-color: rgba(127, 179, 255, 0.3);
	}

	.inline-action {
		padding: 0.45rem 0.75rem;
		font-size: 0.84rem;
	}

	.status-chip {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0.45rem 0.7rem;
		background: rgba(127, 179, 255, 0.14);
		color: #e8f1ff;
		font-size: 0.78rem;
	}

	.badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 1.25rem;
		height: 1.25rem;
		padding: 0 0.35rem;
		margin-left: 0.35rem;
		border-radius: 999px;
		background: #ff7b7b;
		color: #081121;
		font-size: 0.72rem;
	}

	.details-list,
	.plain-list,
	.notifications-list,
	.recent-list {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.details-list,
	.recent-list,
	.notifications-list {
		display: grid;
		gap: 0.6rem;
	}

	.details-list li,
	.recent-list li {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.75rem;
		padding: 0.65rem 0.8rem;
		border-radius: 0.75rem;
		background: rgba(255, 255, 255, 0.04);
	}

	.details-list span {
		color: #9ab8e8;
	}

	.compact-list li {
		align-items: center;
	}

	.compact-listing {
		gap: 0.45rem;
		padding-left: 1rem;
		list-style: disc;
		font-size: 0.9rem;
	}

	.recent-list div {
		display: grid;
		gap: 0.2rem;
		min-width: 0;
	}

	.recent-list span {
		font-size: 0.8rem;
		color: #9ab8e8;
		word-break: break-word;
	}

	.plain-list {
		display: grid;
		gap: 0.55rem;
		padding-left: 1rem;
		list-style: disc;
		color: #dce9ff;
	}

	.viewer-surface {
		position: relative;
		min-height: 0;
		height: 100%;
		border-radius: 0.9rem;
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
		gap: 0.7rem;
		padding: 1.5rem;
		text-align: center;
		color: #edf4ff;
	}

	.viewer-overlay {
		background:
			radial-gradient(circle at center, rgba(127, 179, 255, 0.14), transparent 34%),
			rgba(8, 17, 33, 0.76);
		pointer-events: none;
	}

	.viewer-overlay p {
		max-width: 32rem;
		line-height: 1.55;
		color: #cfddf8;
	}

	.drop-overlay {
		background: rgba(7, 17, 31, 0.76);
		backdrop-filter: blur(8px);
	}

	.drop-overlay div {
		display: grid;
		gap: 0.3rem;
		padding: 1.1rem 1.3rem;
		border-radius: 0.9rem;
		background: rgba(127, 179, 255, 0.14);
		border: 1px solid rgba(127, 179, 255, 0.22);
	}

	.messages-dock {
		position: absolute;
		right: 0.85rem;
		bottom: 0.85rem;
		width: min(24rem, calc(100% - 1.7rem));
		max-height: min(18rem, calc(100% - 1.7rem));
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		gap: 0.75rem;
		opacity: 0;
		transform: translateY(0.75rem);
		pointer-events: none;
		transition:
			opacity 180ms ease,
			transform 180ms ease;
		overflow: hidden;
		z-index: 4;
	}

	.messages-dock.is-visible {
		opacity: 1;
		transform: translateY(0);
		pointer-events: auto;
	}

	.notifications-list {
		overflow: auto;
		padding-right: 0.1rem;
	}

	.notification {
		display: grid;
		gap: 0.25rem;
		padding: 0.75rem 0.85rem;
		border-radius: 0.8rem;
		border: 1px solid transparent;
		font-size: 0.9rem;
	}

	.notification strong {
		text-transform: uppercase;
		letter-spacing: 0.08em;
		font-size: 0.72rem;
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

	.messages-fab {
		position: absolute;
		right: 0.85rem;
		bottom: 0.85rem;
		z-index: 3;
	}

	@media (max-width: 1360px) {
		.topbar {
			grid-template-columns: 1fr;
		}

		.topbar-nav,
		.topbar-actions {
			justify-content: flex-start;
		}

		.home-grid,
		.about-grid,
		.meta-grid,
		.viewer-meta-strip {
			grid-template-columns: 1fr 1fr;
		}
	}

	@media (max-width: 820px) {
		.app-shell {
			padding: 0.6rem;
		}

		.home-screen,
		.about-screen,
		.viewer-screen {
			padding: 0.7rem;
		}

		.home-grid,
		.about-grid,
		.meta-grid,
		.viewer-meta-strip {
			grid-template-columns: 1fr;
		}

		.document-pill {
			width: 100%;
		}

		.messages-dock {
			left: 0.7rem;
			right: 0.7rem;
			width: auto;
			bottom: 0.7rem;
		}

		.messages-fab {
			right: 0.7rem;
			bottom: 0.7rem;
		}
	}
</style>
