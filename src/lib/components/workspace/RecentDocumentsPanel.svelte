<script lang="ts">
	import type { CadRecentDocument } from '$lib/types/cad';

	type RecentDocumentsPanelProps = {
		recentDocuments: CadRecentDocument[];
		isTauriRuntime: boolean;
		isOpening: boolean;
		onOpenRecent: (recentDocument: CadRecentDocument) => void | Promise<void>;
		onClearRecents: () => void | Promise<void>;
	};

	let {
		recentDocuments,
		isTauriRuntime,
		isOpening,
		onOpenRecent,
		onClearRecents
	}: RecentDocumentsPanelProps = $props();

	function handleOpenRecent(recentDocument: CadRecentDocument): void {
		void onOpenRecent(recentDocument);
	}

	function handleClearRecents(): void {
		void onClearRecents();
	}
</script>

<section class="home-panel card-panel">
	<div class="section-header">
		<h2>Desenhos recentes</h2>
		<button class="inline-action" type="button" onclick={handleClearRecents}> Limpar </button>
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
						onclick={() => handleOpenRecent(recentDocument)}
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
