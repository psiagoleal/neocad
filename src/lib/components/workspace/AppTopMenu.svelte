<script lang="ts">
	import type { CadRecentDocument } from '$lib/types/cad';
	import type { WorkspaceView } from '$lib/components/workspace/types';

	type MenuKey = 'file' | 'view' | 'window' | 'help' | null;

	type AppTopMenuProps = {
		appName: string;
		statusLabel: string;
		runtimeLabel: string;
		activeWorkspace: WorkspaceView;
		currentDocumentTitle?: string | null;
		hasVisitedViewerWorkspace: boolean;
		unreadMessages: number;
		isViewerReady: boolean;
		isOpening: boolean;
		recentDocuments: CadRecentDocument[];
		onGoHome: () => void;
		onGoViewer: () => void;
		onGoAbout: () => void;
		onOpenDrawing: () => void | Promise<void>;
		onOpenRecent: (recentDocument: CadRecentDocument) => void | Promise<void>;
		onClearRecents: () => void | Promise<void>;
		onFitView: () => void;
		onToggleBackground: () => void;
		onToggleMessages: () => void;
	};

	let {
		appName,
		statusLabel,
		runtimeLabel,
		activeWorkspace,
		currentDocumentTitle = null,
		hasVisitedViewerWorkspace,
		unreadMessages,
		isViewerReady,
		isOpening,
		recentDocuments,
		onGoHome,
		onGoViewer,
		onGoAbout,
		onOpenDrawing,
		onOpenRecent,
		onClearRecents,
		onFitView,
		onToggleBackground,
		onToggleMessages
	}: AppTopMenuProps = $props();

	let openMenu: MenuKey = $state(null);
	let menuRoot: HTMLElement | null = $state(null);

	function toggleMenu(menu: Exclude<MenuKey, null>): void {
		openMenu = openMenu === menu ? null : menu;
	}

	function closeMenus(): void {
		openMenu = null;
	}

	function handleDocumentClick(event: MouseEvent): void {
		if (menuRoot == null || !(event.target instanceof Node)) {
			return;
		}

		if (!menuRoot.contains(event.target)) {
			closeMenus();
		}
	}

	function handleDocumentKeydown(event: KeyboardEvent): void {
		if (event.key === 'Escape') {
			closeMenus();
		}
	}

	function runAction(action: () => void | Promise<void>): void {
		closeMenus();
		void action();
	}

	function handleOpenRecent(recentDocument: CadRecentDocument): void {
		closeMenus();
		void onOpenRecent(recentDocument);
	}
</script>

<svelte:document onclick={handleDocumentClick} onkeydown={handleDocumentKeydown} />

<header class="top-menu card-panel" bind:this={menuRoot}>
	<div class="top-menu-bar">
		<div class="menu-brand">
			<p class="eyebrow">NeoCAD Workspace</p>
			<h1>{appName}</h1>
		</div>

		<nav class="menu-groups" aria-label="Menu principal do workspace">
			<div class="menu-group">
				<button
					class:active={openMenu === 'file'}
					class="menu-trigger"
					type="button"
					onclick={() => toggleMenu('file')}
				>
					Arquivo
				</button>

				{#if openMenu === 'file'}
					<div class="menu-dropdown">
						<button class="menu-item" type="button" onclick={() => runAction(onOpenDrawing)}>
							{isOpening ? 'Abrindo desenho...' : 'Abrir desenho CAD'}
						</button>

						<div class="menu-divider"></div>
						<p class="menu-caption">Recentes</p>

						{#if recentDocuments.length > 0}
							<div class="menu-items-scroll">
								{#each recentDocuments.slice(0, 6) as recentDocument (recentDocument.openedAt + recentDocument.fileName)}
									<button
										class="menu-item menu-item-secondary"
										type="button"
										onclick={() => handleOpenRecent(recentDocument)}
										disabled={recentDocument.path == null || isOpening}
									>
										{recentDocument.fileName}
									</button>
								{/each}
							</div>
						{:else}
							<p class="menu-empty-copy">Nenhum desenho recente registrado.</p>
						{/if}

						<div class="menu-divider"></div>
						<button
							class="menu-item menu-item-secondary"
							type="button"
							onclick={() => runAction(onClearRecents)}
						>
							Limpar recentes
						</button>
					</div>
				{/if}
			</div>

			<div class="menu-group">
				<button
					class:active={openMenu === 'view'}
					class="menu-trigger"
					type="button"
					onclick={() => toggleMenu('view')}
				>
					Exibir
				</button>

				{#if openMenu === 'view'}
					<div class="menu-dropdown">
						<button class="menu-item" type="button" onclick={() => runAction(onFitView)}>
							Ajustar vista ao desenho
						</button>
						<button class="menu-item" type="button" onclick={() => runAction(onToggleBackground)}>
							Alternar fundo do canvas
						</button>
						<button class="menu-item" type="button" onclick={() => runAction(onToggleMessages)}>
							Mostrar ou ocultar mensagens
						</button>
					</div>
				{/if}
			</div>

			<div class="menu-group">
				<button
					class:active={openMenu === 'window'}
					class="menu-trigger"
					type="button"
					onclick={() => toggleMenu('window')}
				>
					Janela
				</button>

				{#if openMenu === 'window'}
					<div class="menu-dropdown">
						<button
							class:active={activeWorkspace === 'home'}
							class="menu-item"
							type="button"
							onclick={() => runAction(onGoHome)}
						>
							Integração do viewer
						</button>
						<button
							class:active={activeWorkspace === 'viewer'}
							class="menu-item"
							type="button"
							onclick={() => runAction(onGoViewer)}
							disabled={!hasVisitedViewerWorkspace && currentDocumentTitle == null}
						>
							Canvas CAD
						</button>
						<button
							class:active={activeWorkspace === 'about'}
							class="menu-item"
							type="button"
							onclick={() => runAction(onGoAbout)}
						>
							Sobre
						</button>
					</div>
				{/if}
			</div>

			<div class="menu-group">
				<button
					class:active={openMenu === 'help'}
					class="menu-trigger"
					type="button"
					onclick={() => toggleMenu('help')}
				>
					Ajuda
				</button>

				{#if openMenu === 'help'}
					<div class="menu-dropdown">
						<button class="menu-item" type="button" onclick={() => runAction(onGoAbout)}>
							Sobre o NeoCAD
						</button>
					</div>
				{/if}
			</div>
		</nav>

		<div class="top-menu-actions">
			<span class="status-chip">{statusLabel}</span>
			<span class="status-chip">Runtime {runtimeLabel}</span>
			<span class="status-chip">Viewer {isViewerReady ? 'pronto' : 'inicializando'}</span>
			<button class="utility-button" type="button" onclick={onToggleMessages}>
				Mensagens
				{#if unreadMessages > 0}
					<span class="badge">{unreadMessages}</span>
				{/if}
			</button>
		</div>
	</div>

	<div class="top-menu-meta">
		<div class="workspace-tabs" aria-label="Navegação rápida do workspace">
			<button
				class:active={activeWorkspace === 'home'}
				class="nav-button"
				type="button"
				onclick={onGoHome}
			>
				Integração do viewer
			</button>
			<button
				class:active={activeWorkspace === 'viewer'}
				class="nav-button"
				type="button"
				onclick={onGoViewer}
				disabled={!hasVisitedViewerWorkspace && currentDocumentTitle == null}
			>
				Canvas CAD
			</button>
			<button
				class:active={activeWorkspace === 'about'}
				class="nav-button"
				type="button"
				onclick={onGoAbout}
			>
				Sobre
			</button>
		</div>

		<div class="top-menu-current-document">
			<span class="label">Documento</span>
			<strong>{currentDocumentTitle ?? 'Nenhum desenho aberto'}</strong>
		</div>
	</div>
</header>
