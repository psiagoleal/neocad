<!-- Caminho relativo: src/lib/components/workspace/CommandLine.svelte -->
<script lang="ts">
	/**
	 * Linha de comando própria (spec 08), abaixo do canvas: prompt, histórico,
	 * sugestões e as teclas Enter, Espaço, Esc e setas. A lógica mora no motor
	 * (`$lib/command-line/motor`); aqui ficam só a tela e o teclado.
	 */
	import { onMount, tick } from 'svelte';
	import { CATALOGO_DA_SPEC } from '$lib/command-line/catalogo';
	import { COMANDOS } from '$lib/command-line/comandos';
	import {
		MotorDeComandos,
		mensagemNaoEditavel,
		type DocumentoDosComandos,
		type Escolha,
		type Mensagem
	} from '$lib/command-line/motor';
	import { formatarPonto } from '$lib/command-line/pontos';
	import type { CadEntityId, CadPoint } from '$lib/types/cad';

	type CommandLineProps = {
		documento: () => DocumentoDosComandos | null;
		aposEditar: () => void;
		/** `Esc` sem comando em curso também esvazia a seleção (spec 08, C7). */
		esvaziarSelecao: () => void;
		/** Ids selecionados na tela, para os comandos que trabalham sobre a seleção. */
		selecionados: () => CadEntityId[];
		/** Todas as entidades do espaço mostrado (o "<selecionar todos>"). */
		todas: () => CadEntityId[];
		/** Liga (com quem recebe o clique) ou desliga o modo de escolha da tela. */
		definirEscolha: (aoEscolher: ((escolha: Escolha) => void) | null) => void;
		/** Liga (com quem recebe o ponto) ou desliga o modo de ponto da tela. */
		definirPonto: (aoApontar: ((ponto: CadPoint) => void) | null) => void;
		/** Recebe quem escreve no histórico o aviso de entidade não editável (spec 07, V7). */
		definirAvisoNaoEditavel: (aviso: (tipo: string) => void) => void;
	};

	let {
		documento,
		aposEditar,
		esvaziarSelecao,
		selecionados,
		todas,
		definirEscolha,
		definirPonto,
		definirAvisoNaoEditavel
	}: CommandLineProps = $props();

	/** Linhas guardadas no histórico; as mais antigas saem. */
	const LIMITE_DO_HISTORICO = 200;

	let historico: Mensagem[] = $state([]);
	let texto = $state('');
	let prompt = $state('Comando:');
	let historicoEl: HTMLUListElement | null = $state(null);

	/** Textos confirmados, para as setas (o mais recente no fim). */
	const recentes: string[] = [];
	let indiceRecente = -1;

	const motor = new MotorDeComandos({
		catalogo: CATALOGO_DA_SPEC,
		comandos: COMANDOS,
		documento: () => documento(),
		aposEditar: () => aposEditar(),
		selecao: {
			ids: () => selecionados(),
			esvaziar: () => esvaziarSelecao(),
			todas: () => todas()
		},
		mensagem: (m) => {
			historico = [...historico, m].slice(-LIMITE_DO_HISTORICO);
		}
	});

	/**
	 * Depois de cada passo: o prompt novo e o modo do clique na tela. O ponto
	 * clicado entra como texto, como se tivesse sido digitado (spec 10, P1, P3); o
	 * objeto clicado vai ao comando com o ponto (T4).
	 */
	function sincronizar(): void {
		prompt = motor.prompt;
		definirPonto(null);
		definirEscolha(null);
		if (motor.querPonto) {
			definirPonto((ponto) => confirmar(formatarPonto(ponto)));
		} else if (motor.querEscolha) {
			definirEscolha((escolha) => {
				motor.escolher(escolha);
				sincronizar();
			});
		}
		void rolarHistorico();
	}

	// O clique em entidade só do upstream, fora de comando, responde no histórico (V7).
	onMount(() =>
		definirAvisoNaoEditavel((tipo) => {
			historico = [...historico, { tipo: 'erro' as const, texto: mensagemNaoEditavel(tipo) }].slice(
				-LIMITE_DO_HISTORICO
			);
			void rolarHistorico();
		})
	);

	const sugestoes = $derived.by(() => {
		void prompt;
		return motor.sugerir(texto);
	});

	async function rolarHistorico(): Promise<void> {
		await tick();
		historicoEl?.scrollTo({ top: historicoEl.scrollHeight });
	}

	function confirmar(valor: string): void {
		if (valor.trim() !== '') {
			recentes.push(valor);
			if (recentes.length > LIMITE_DO_HISTORICO) recentes.shift();
		}
		indiceRecente = -1;
		texto = '';
		motor.confirmar(valor);
		sincronizar();
	}

	function navegarRecentes(passo: -1 | 1): void {
		if (recentes.length === 0) return;
		const base = indiceRecente === -1 ? recentes.length : indiceRecente;
		const novo = Math.min(recentes.length, Math.max(0, base + passo));
		indiceRecente = novo === recentes.length ? -1 : novo;
		texto = indiceRecente === -1 ? '' : recentes[indiceRecente];
	}

	/**
	 * `Delete` fora de campo de texto, sem comando em curso e com seleção, apaga
	 * como o ERASE (spec 10, E3).
	 */
	function aoTeclarNoDocumento(evento: KeyboardEvent): void {
		const alvo = evento.target as HTMLElement | null;
		const emCampo =
			alvo?.tagName === 'INPUT' || alvo?.tagName === 'TEXTAREA' || alvo?.isContentEditable;
		if (evento.key !== 'Delete' || emCampo || motor.emCurso != null) return;
		if (selecionados().length === 0) return;
		evento.preventDefault();
		confirmar('_ERASE');
	}

	function aoTeclar(evento: KeyboardEvent): void {
		switch (evento.key) {
			case 'Enter':
			case ' ':
				// Como no AutoCAD, o Espaço confirma; na linha vazia, repete (C6).
				evento.preventDefault();
				confirmar(texto);
				break;
			case 'Escape':
				evento.preventDefault();
				texto = '';
				if (motor.emCurso == null) esvaziarSelecao();
				motor.cancelar();
				sincronizar();
				break;
			case 'ArrowUp':
				evento.preventDefault();
				navegarRecentes(-1);
				break;
			case 'ArrowDown':
				evento.preventDefault();
				navegarRecentes(1);
				break;
		}
	}
</script>

<svelte:document onkeydown={aoTeclarNoDocumento} />

<section class="linha-de-comando">
	<ul class="historico" role="log" aria-label="Histórico de comandos" bind:this={historicoEl}>
		{#each historico as mensagem, indice (indice)}
			<li class={`mensagem-${mensagem.tipo}`}>{mensagem.texto}</li>
		{/each}
	</ul>

	<div class="entrada">
		{#if texto.trim() !== '' && sugestoes.length > 0}
			<ul class="sugestoes" role="listbox" aria-label="Sugestões">
				{#each sugestoes as comando (comando.en)}
					<li role="option" aria-selected="false">
						<button type="button" onclick={() => confirmar(comando.en)}>
							<strong>{comando.en}</strong>
							<span>{comando.ptbr}</span>
							<span class="apelidos">{comando.apelidos.join(', ')}</span>
						</button>
					</li>
				{/each}
			</ul>
		{:else}
			<ul class="sugestoes vazia" role="listbox" aria-label="Sugestões"></ul>
		{/if}

		<label>
			<span class="prompt">{prompt}</span>
			<input
				type="text"
				aria-label="Linha de comando"
				data-prompt={prompt}
				autocomplete="off"
				spellcheck="false"
				bind:value={texto}
				onkeydown={aoTeclar}
			/>
		</label>
	</div>
</section>

<style>
	.linha-de-comando {
		margin-top: var(--space-2, 0.5rem);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md, 6px);
		background: #0b1628;
		color: #cfddf8;
		font-family: ui-monospace, 'SFMono-Regular', Menlo, Consolas, monospace;
		font-size: 0.85rem;
	}

	.historico {
		list-style: none;
		margin: 0;
		padding: 0.3rem 0.6rem;
		height: 4.6em;
		overflow-y: auto;
	}

	.mensagem-eco {
		color: #9fb2d3;
	}

	.mensagem-erro {
		color: #ffb4a8;
	}

	.entrada {
		position: relative;
		border-top: 1px solid var(--color-border);
	}

	label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.3rem 0.6rem;
	}

	.prompt {
		white-space: nowrap;
		color: #fff;
	}

	input {
		flex: 1;
		min-width: 0;
		border: none;
		outline: none;
		background: transparent;
		color: inherit;
		font: inherit;
	}

	.sugestoes {
		position: absolute;
		left: 0.6rem;
		bottom: 100%;
		z-index: 2;
		list-style: none;
		margin: 0;
		padding: 0.2rem 0;
		min-width: 18rem;
		background: #10203a;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm, 4px);
	}

	.sugestoes.vazia {
		display: none;
	}

	.sugestoes button {
		display: flex;
		gap: 0.8rem;
		width: 100%;
		padding: 0.2rem 0.6rem;
		border: none;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.sugestoes button:hover {
		background: #1d3b66;
	}

	.apelidos {
		margin-left: auto;
		color: #9fb2d3;
	}
</style>
