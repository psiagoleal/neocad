<script lang="ts">
	import { onMount } from 'svelte';
	import AboutScreen from '$lib/components/workspace/AboutScreen.svelte';
	import AppTopMenu from '$lib/components/workspace/AppTopMenu.svelte';
	import HelpCommandsDialog from '$lib/components/workspace/HelpCommandsDialog.svelte';
	import HomeScreen from '$lib/components/workspace/HomeScreen.svelte';
	import MessagesDock from '$lib/components/workspace/MessagesDock.svelte';
	import ViewerScreen from '$lib/components/workspace/ViewerScreen.svelte';
	import type { WorkspaceView } from '$lib/components/workspace/types';
	import { appMetadata, nextMilestones, primaryStack, supportedTargets } from '$lib/config/app';
	import {
		createCadDocumentPayloadFromFile,
		getCadRuntimeLabel,
		readCadDocumentFromPath,
		selectCadDocument
	} from '$lib/services/cad-file';
	import { listCadCommandCatalog } from '$lib/services/cad-commands';
	import { CadDocument } from '$lib/services/cad-document';
	import {
		clearRecentDocuments as clearStoredRecentDocuments,
		listRecentDocuments,
		registerRecentDocument
	} from '$lib/services/recent-documents';
	import type {
		CadCommandCatalogItem,
		CadDocumentPayload,
		CadHistoryState,
		CadRecentDocument,
		CadViewerDocumentState,
		CadViewerMessage,
		CadViewerProgressState
	} from '$lib/types/cad';
	import { NeoCadViewer } from '$lib/viewer/neocad-viewer';

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
	let isCommandsHelpOpen = $state(false);
	let commandCatalog: CadCommandCatalogItem[] = $state([]);

	/** Documento do kernel próprio. Convive com o upstream durante a transição. */
	let kernelDocument: CadDocument | null = $state(null);
	const emptyHistory: CadHistoryState = {
		canUndo: false,
		canRedo: false,
		undoLabel: null,
		redoLabel: null,
		undoDepth: 0,
		redoDepth: 0
	};
	let history: CadHistoryState = $state(emptyHistory);

	/**
	 * Relê o estado da pilha após qualquer ação que possa tê-lo mudado.
	 *
	 * O kernel é a fonte de verdade; guardar uma cópia derivada aqui e mantê-la
	 * sincronizada à mão seria a forma mais fácil de o menu passar a mentir.
	 */
	function refreshHistory(): void {
		history = kernelDocument?.getHistory() ?? emptyHistory;
	}

	/**
	 * Carrega no kernel o desenho que o upstream acabou de abrir.
	 *
	 * O upstream continua sendo quem lê o arquivo e quem desenha (K5 e K6 ainda
	 * não chegaram); o kernel passa a ser a fonte de verdade sobre o que existe
	 * no desenho.
	 */
	async function loadIntoKernel(): Promise<void> {
		if (viewerController == null) {
			return;
		}

		const snapshot = viewerController.extractDocumentSnapshot();

		if (snapshot == null) {
			return;
		}

		try {
			kernelDocument ??= await CadDocument.create();
			const report = kernelDocument.load(snapshot);
			refreshHistory();

			pushNotification(
				'info',
				`Kernel: ${report.entityCount} entidade(s) em ${report.layerCount} camada(s).` +
					(report.unsupportedCount > 0
						? ` ${report.unsupportedCount} entidade(s) ainda não suportada(s) pelo kernel.`
						: '')
			);
		} catch (error) {
			pushNotification(
				'warning',
				error instanceof Error
					? `Kernel não pôde carregar o desenho: ${error.message}`
					: 'Kernel não pôde carregar o desenho.'
			);
		}
	}

	async function undoAction(): Promise<void> {
		try {
			if (kernelDocument?.undo() !== true) {
				return;
			}

			refreshHistory();
		} catch (error) {
			pushNotification(
				'error',
				error instanceof Error ? error.message : 'Falha ao desfazer a última ação.'
			);
		}
	}

	async function redoAction(): Promise<void> {
		try {
			if (kernelDocument?.redo() !== true) {
				return;
			}

			refreshHistory();
		} catch (error) {
			pushNotification(
				'error',
				error instanceof Error ? error.message : 'Falha ao refazer a última ação.'
			);
		}
	}

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

	function handleViewerHostReady(element: HTMLDivElement): void {
		viewerHost = element;
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

	function handleViewerDragEnter(event: DragEvent): void {
		preventFileDropDefaults(event);
		isDragActive = true;
	}

	function handleViewerDragOver(event: DragEvent): void {
		preventFileDropDefaults(event);
	}

	function handleViewerDragLeave(event: DragEvent): void {
		preventFileDropDefaults(event);
		if (event.currentTarget === event.target) {
			isDragActive = false;
		}
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

	function showCommandsHelp(): void {
		if (!isViewerReady || viewerController == null) {
			pushNotification(
				'info',
				'O catálogo de comandos fica disponível após a inicialização do viewer.'
			);
			return;
		}

		commandCatalog = listCadCommandCatalog(viewerController);
		isCommandsHelpOpen = true;
	}

	function closeCommandsHelp(): void {
		isCommandsHelpOpen = false;
	}

	onMount(() => {
		void refreshRecentDocuments();

		let controller: NeoCadViewer | null = null;
		let isDisposed = false;

		queueMicrotask(() => {
			if (isDisposed) {
				return;
			}

			if (viewerHost == null) {
				pushNotification('error', 'O container principal do viewer não foi inicializado.');
				return;
			}

			controller = new NeoCadViewer({
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
					void loadIntoKernel();
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
		});

		return () => {
			isDisposed = true;
			isViewerReady = false;
			currentDocument = null;
			progress = null;
			viewerController = null;
			kernelDocument = null;
			history = emptyHistory;

			if (controller != null) {
				void controller.destroy();
			}
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
	<AppTopMenu
		appName={appMetadata.name}
		{activeWorkspace}
		currentDocumentTitle={currentDocument?.fileName ?? null}
		{hasVisitedViewerWorkspace}
		{unreadMessages}
		{isOpening}
		{recentDocuments}
		{history}
		onUndo={undoAction}
		onRedo={redoAction}
		onGoHome={showHomeWorkspace}
		onGoViewer={() => showViewerWorkspace()}
		onGoAbout={showAboutWorkspace}
		onOpenDrawing={openCadDrawing}
		onOpenRecent={openRecentDrawing}
		onClearRecents={clearRecentDocumentsList}
		onFitView={fitDrawingToView}
		onToggleBackground={toggleViewerBackground}
		onToggleMessages={toggleMessagesDock}
		onShowCommands={showCommandsHelp}
	/>

	<div class="workspace-stage">
		<section
			class:workspace-active={activeWorkspace === 'home'}
			class="workspace-screen home-screen"
			aria-hidden={activeWorkspace !== 'home'}
		>
			<HomeScreen
				{runtimeLabel}
				{isViewerReady}
				{isMessagesVisible}
				{isOpening}
				{isTauriRuntime}
				{recentDocuments}
				onOpenDrawing={openCadDrawing}
				onEnterViewer={() => showViewerWorkspace(true)}
				onOpenRecent={openRecentDrawing}
				onClearRecents={clearRecentDocumentsList}
			/>
		</section>

		<section
			class:workspace-active={activeWorkspace === 'viewer'}
			class="workspace-screen viewer-screen"
			aria-hidden={activeWorkspace !== 'viewer'}
		>
			<ViewerScreen
				{currentDocument}
				{backgroundTheme}
				{progress}
				{isViewerReady}
				{isOpening}
				{isDragActive}
				{isTauriRuntime}
				onOpenDrawing={openCadDrawing}
				onFitView={fitDrawingToView}
				onToggleBackground={toggleViewerBackground}
				onDragEnter={handleViewerDragEnter}
				onDragOver={handleViewerDragOver}
				onDragLeave={handleViewerDragLeave}
				onDrop={handleFileDrop}
				onViewerHostReady={handleViewerHostReady}
			/>
		</section>

		<section
			class:workspace-active={activeWorkspace === 'about'}
			class="workspace-screen about-screen"
			aria-hidden={activeWorkspace !== 'about'}
		>
			<AboutScreen
				appName={appMetadata.name}
				status={appMetadata.status}
				license={appMetadata.license}
				{runtimeLabel}
				{primaryStack}
				{nextMilestones}
				{supportedTargets}
			/>
		</section>

		<MessagesDock
			{notifications}
			isVisible={isMessagesVisible}
			{unreadMessages}
			onOpen={openMessagesDock}
			onClose={closeMessagesDock}
		/>

		<HelpCommandsDialog
			open={isCommandsHelpOpen}
			commands={commandCatalog}
			onClose={closeCommandsHelp}
		/>
	</div>
</section>
