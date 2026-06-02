<script lang="ts">
	import {
		CAD_COMMAND_CATEGORY_LABELS,
		CAD_COMMAND_CATEGORY_ORDER
	} from '$lib/config/cad-command-catalog';
	import type { CadCommandCatalogItem, CadCommandCategory } from '$lib/types/cad';

	type HelpCommandsDialogProps = {
		open: boolean;
		commands: CadCommandCatalogItem[];
		onClose: () => void;
	};

	let { open, commands, onClose }: HelpCommandsDialogProps = $props();

	let filter = $state('');

	const normalizedFilter = $derived(filter.trim().toLowerCase());

	const filteredCommands = $derived(
		normalizedFilter.length === 0
			? commands
			: commands.filter((command) =>
					`${command.label} ${command.command} ${command.notes ?? ''}`
						.toLowerCase()
						.includes(normalizedFilter)
				)
	);

	const groupedCommands = $derived(
		CAD_COMMAND_CATEGORY_ORDER.map((category) => ({
			category,
			label: CAD_COMMAND_CATEGORY_LABELS[category],
			items: filteredCommands.filter((command) => command.category === category)
		})).filter((group) => group.items.length > 0)
	);

	function categoryLabel(category: CadCommandCategory): string {
		return CAD_COMMAND_CATEGORY_LABELS[category];
	}

	function handleBackdropKeydown(event: KeyboardEvent): void {
		if (event.key === 'Escape') {
			onClose();
		}
	}
</script>

{#if open}
	<div
		class="command-dialog-backdrop"
		role="presentation"
		onclick={onClose}
		onkeydown={handleBackdropKeydown}
	>
		<div
			class="command-dialog card-panel"
			role="dialog"
			aria-modal="true"
			aria-label="Referência de comandos CAD"
			tabindex="-1"
			onclick={(event) => event.stopPropagation()}
			onkeydown={handleBackdropKeydown}
		>
			<header class="command-dialog-header compact-header">
				<div>
					<p class="eyebrow">Ajuda</p>
					<h2>Comandos CAD</h2>
				</div>
				<button class="inline-action" type="button" onclick={onClose}>Fechar</button>
			</header>

			<p class="command-dialog-hint">
				Lista derivada em tempo de execução dos comandos aceitos pelo viewer. Digite-os na barra de
				comandos do canvas.
			</p>

			<div class="command-dialog-filter">
				<input
					type="search"
					placeholder="Filtrar comandos..."
					bind:value={filter}
					aria-label="Filtrar comandos"
				/>
				<span class="command-dialog-count">{filteredCommands.length} de {commands.length}</span>
			</div>

			<div class="command-dialog-body">
				{#if commands.length === 0}
					<p class="empty-copy">
						Nenhum comando disponível. Inicialize o viewer abrindo um desenho CAD.
					</p>
				{:else if groupedCommands.length === 0}
					<p class="empty-copy">Nenhum comando corresponde ao filtro "{filter}".</p>
				{:else}
					{#each groupedCommands as group (group.category)}
						<section class="command-group">
							<h3 class="command-group-title">{categoryLabel(group.category)}</h3>
							<ul class="command-list">
								{#each group.items as command (command.id)}
									<li class="command-row">
										<div class="command-row-main">
											<span class="command-name">{command.label}</span>
											<code class="command-token">{command.command}</code>
										</div>
										{#if command.notes}
											<span class="command-notes">{command.notes}</span>
										{/if}
									</li>
								{/each}
							</ul>
						</section>
					{/each}
				{/if}
			</div>
		</div>
	</div>
{/if}
