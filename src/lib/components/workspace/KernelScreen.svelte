<!-- Caminho relativo: src/lib/components/workspace/KernelScreen.svelte -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { TelaDoKernel } from '$lib/render/tela-do-kernel';

	type KernelScreenProps = {
		/** Verdadeiro quando o desenho aberto foi lido pelo kernel e está na tela. */
		visible: boolean;
		dark: boolean;
		onReady: (tela: TelaDoKernel) => void;
		onError: (message: string) => void;
	};

	let { visible, dark, onReady, onError }: KernelScreenProps = $props();

	let canvas: HTMLCanvasElement | null = $state(null);
	let tela: TelaDoKernel | null = $state(null);
	let espacos: string[] = $state([]);
	let atual = $state('');

	onMount(() => {
		if (canvas == null) return;

		try {
			tela = new TelaDoKernel(canvas, {
				escuro: dark,
				aoMudarEspacos: (lista, espaco) => {
					espacos = lista;
					atual = espaco;
				}
			});
			onReady(tela);
		} catch (error) {
			onError(error instanceof Error ? error.message : String(error));
		}

		return () => tela?.destruir();
	});

	$effect(() => {
		tela?.definirFundo(dark);
	});
</script>

<div class="kernel-screen" class:kernel-screen-visible={visible}>
	<canvas bind:this={canvas} data-tela="kernel" aria-label="Desenho"></canvas>

	{#if visible && espacos.length > 1}
		<div class="kernel-screen-tabs" role="tablist" aria-label="Espaços do desenho">
			{#each espacos as espaco (espaco)}
				<button
					type="button"
					role="tab"
					aria-selected={espaco === atual}
					onclick={() => tela?.mostrarEspaco(espaco)}
				>
					{espaco}
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.kernel-screen {
		position: absolute;
		inset: 0;
		visibility: hidden;
		pointer-events: none;
	}

	.kernel-screen-visible {
		visibility: visible;
		pointer-events: auto;
	}

	canvas {
		display: block;
		width: 100%;
		height: 100%;
		touch-action: none;
	}

	.kernel-screen-tabs {
		position: absolute;
		left: 0.5rem;
		bottom: 0.5rem;
		display: flex;
		gap: 0.25rem;
	}

	.kernel-screen-tabs button {
		padding: 0.2rem 0.6rem;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm, 4px);
		background: rgba(8, 17, 33, 0.85);
		color: #cfddf8;
		font: inherit;
		font-size: 0.8rem;
		cursor: pointer;
	}

	.kernel-screen-tabs button[aria-selected='true'] {
		background: #1d3b66;
		color: #fff;
	}
</style>
