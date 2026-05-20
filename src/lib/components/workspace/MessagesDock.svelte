<script lang="ts">
	import type { CadViewerMessage } from '$lib/types/cad';

	type MessagesDockProps = {
		notifications: CadViewerMessage[];
		isVisible: boolean;
		unreadMessages: number;
		onOpen: () => void;
		onClose: () => void;
	};

	let { notifications, isVisible, unreadMessages, onOpen, onClose }: MessagesDockProps = $props();
</script>

<section class:is-visible={isVisible} class="messages-dock card-panel">
	<header class="messages-header compact-header">
		<div>
			<p class="eyebrow">Mensagens</p>
			<h2>Status da integração</h2>
		</div>
		<button class="inline-action" type="button" onclick={onClose}>Ocultar</button>
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

{#if !isVisible}
	<button class="messages-fab" type="button" onclick={onOpen}>
		Mensagens
		{#if unreadMessages > 0}
			<span class="badge">{unreadMessages}</span>
		{/if}
	</button>
{/if}
