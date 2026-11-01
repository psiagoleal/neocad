// Caminho relativo: src/lib/render/tela-do-kernel.ts

/**
 * \file src/lib/render/tela-do-kernel.ts
 * \brief Liga o documento do kernel a um canvas: blocos, vista, pan e zoom.
 * \author Iago Leal
 * \date 2026-11-01
 *
 * O kernel prepara os blocos de segmentos (spec 06, W8); aqui eles vão ao
 * desenhista com a transformação da vista, e nada mais é calculado (ADR 0008).
 * O canvas expõe o que desenhou no último quadro como atributos, que são o que
 * os testes observam (spec 07, "O que os testes observam na tela").
 *
 * O caminho é escolhido uma vez, ao criar: WebGL2 se o navegador oferece; se
 * não, canvas 2D (spec 00, N4).
 */

import type { CadDocument } from '$lib/services/cad-document';
import type { CadLayerId } from '$lib/types/cad';
import { DesenhistaCanvas2d } from './canvas2d';
import { corDaCamada, type Rgb } from './cor';
import type { Desenhista } from './desenhista';
import { DesenhistaWebgl2 } from './webgl2';
import {
	ampliar,
	deslocar,
	enquadrar,
	erroDeCorda,
	fatorDaRoda,
	redimensionar,
	transformacao,
	type Vista
} from './vista';

const FUNDO_ESCURO: Rgb = [8, 17, 33];
const FUNDO_CLARO: Rgb = [246, 248, 251];

/** Botão do meio do mouse, o do pan (spec 07). */
const BOTAO_DO_MEIO = 1;

export type AoMudarEspacos = (espacos: string[], atual: string) => void;

/**
 * Vista do quadro desenhado, para quem desenha por baixo seguir: centro em
 * coordenadas do desenho e escala em pixels por unidade. `null` quando a tela
 * está opaca e não há o que seguir.
 */
export type AoMudarVista = (vista: { x: number; y: number; escala: number } | null) => void;

/** Escolhe o caminho de desenho do canvas: GPU se houver, reserva se não. */
export function criarDesenhista(canvas: HTMLCanvasElement): Desenhista {
	const gl = canvas.getContext('webgl2', { antialias: true, preserveDrawingBuffer: false });
	if (gl != null) {
		return new DesenhistaWebgl2(gl);
	}

	const ctx = canvas.getContext('2d');
	if (ctx == null) {
		throw new Error('O navegador não oferece WebGL2 nem canvas 2D.');
	}
	return new DesenhistaCanvas2d(ctx);
}

export class TelaDoKernel {
	private readonly desenhista: Desenhista;
	private documento: CadDocument | null = null;
	private espacos: string[] = [];
	private espaco = '';
	private origem = { x: 0, y: 0 };
	private vista: Vista;
	private entidades = 0;
	/** Caixa do espaço corrente como o atributo a mostra; vazia sem entidades. */
	private caixa = '';
	private quadros = 0;
	private pedido = 0;
	private escuro: boolean;
	private arrasto: { x: number; y: number; id: number } | null = null;
	private aviso: ((mensagem: string) => void) | null = null;
	/** Último aviso de não exibidas dado por espaço, para não repetir. */
	private readonly avisados = new Map<string, string>();
	/** Quem desenha por baixo o que o kernel não modela (o upstream) e segue a vista. */
	private aoMudarVista: AoMudarVista | null = null;
	private readonly observador: ResizeObserver | null;
	private readonly soltar: Array<() => void> = [];

	constructor(
		private readonly canvas: HTMLCanvasElement,
		opcoes: { escuro: boolean; aoMudarEspacos?: AoMudarEspacos }
	) {
		this.escuro = opcoes.escuro;
		this.aoMudarEspacos = opcoes.aoMudarEspacos;
		this.desenhista = criarDesenhista(canvas);
		this.vista = enquadrar(null, canvas.clientWidth || 1, canvas.clientHeight || 1);
		canvas.dataset.caminho = this.desenhista.caminho;
		canvas.dataset.quadros = '0';

		this.ouvir('pointerdown', (e) => this.aoPressionar(e as PointerEvent));
		this.ouvir('pointermove', (e) => this.aoMover(e as PointerEvent));
		this.ouvir('pointerup', (e) => this.aoSoltar(e as PointerEvent));
		this.ouvir('pointercancel', (e) => this.aoSoltar(e as PointerEvent));
		this.ouvir('wheel', (e) => this.aoRolar(e as WheelEvent), { passive: false });
		// O botão do meio abre a rolagem automática em alguns navegadores.
		this.ouvir('mousedown', (e) => {
			if ((e as MouseEvent).button === BOTAO_DO_MEIO) e.preventDefault();
		});

		this.observador =
			typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(() => this.ajustarArea());
		this.observador?.observe(canvas);
		this.ajustarArea();
	}

	private readonly aoMudarEspacos?: AoMudarEspacos;

	/** Caminho escolhido ao criar a tela. */
	get caminho(): Desenhista['caminho'] {
		return this.desenhista.caminho;
	}

	/** Prepara e mostra o documento, começando pelo espaço-modelo. */
	mostrar(documento: CadDocument): void {
		this.documento = documento;
		this.avisados.clear();
		documento.prepareScreen(erroDeCorda(documento.getBounds()));
		this.espacos = documento.getScreenSpaces();
		this.mostrarEspaco(this.espacos[0] ?? '');
	}

	/** Troca o espaço mostrado (modelo ou um layout) e enquadra. */
	mostrarEspaco(espaco: string): void {
		const documento = this.documento;
		if (documento == null || !this.espacos.includes(espaco)) return;

		this.espaco = espaco;
		this.limparBlocos();
		this.origem = documento.getScreenOrigin(espaco);
		this.carregarCamadas(documento.getScreenLayers(espaco));
		this.enquadrar();
		this.aoMudarEspacos?.(this.espacos, espaco);
		this.avisarNaoExibidas();
	}

	/** Enquadra o espaço corrente inteiro. */
	enquadrar(): void {
		const caixa = this.documento?.getScreenExtents(this.espaco) ?? null;
		this.vista = enquadrar(caixa, this.vista.largura, this.vista.altura);
		this.pedirQuadro();
	}

	/** Busca de novo os blocos que a última operação do kernel mudou. */
	atualizar(): void {
		const documento = this.documento;
		if (documento == null) return;

		const mudados = documento.updateScreen().filter((b) => b.space === this.espaco);
		if (mudados.length > 0) {
			const visiveis = new Set(documento.getScreenLayers(this.espaco));
			const cores = this.coresDasCamadas();
			for (const { layer } of mudados) {
				const chave = this.chave(layer);
				if (visiveis.has(layer)) {
					this.desenhista.definirBloco(
						chave,
						documento.getScreenBlock(this.espaco, layer),
						cores.get(layer) ?? this.traco()
					);
					this.chaves.add(chave);
				} else {
					this.desenhista.removerBloco(chave);
					this.chaves.delete(chave);
				}
			}
		}
		this.entidades = documento.getScreenEntityCount(this.espaco);
		this.caixa = this.caixaDoEspaco();
		this.avisarNaoExibidas();
		this.pedirQuadro();
	}

	/**
	 * Põe esta tela sobre outra que desenha o que o kernel não modela, e que
	 * segue a vista pelo aviso; `null` tira a sobreposição. Sobreposto, o
	 * espaço-modelo sai com fundo transparente e a vista é avisada a cada quadro;
	 * nos layouts a tela fica opaca, porque a de baixo só mostra o modelo.
	 */
	sobrepor(aoMudarVista: AoMudarVista | null): void {
		this.aoMudarVista = aoMudarVista;
		this.pedirQuadro();
	}

	/** Para onde vai o aviso das entidades que a tela não exibe (spec 07, V6). */
	avisarCom(aviso: (mensagem: string) => void): void {
		this.aviso = aviso;
	}

	/** Troca o fundo; a cor 7 acompanha (branca no escuro, preta no claro). */
	definirFundo(escuro: boolean): void {
		if (escuro === this.escuro) return;
		this.escuro = escuro;
		if (this.documento != null) {
			this.limparBlocos();
			this.carregarCamadas(this.documento.getScreenLayers(this.espaco));
		}
		this.pedirQuadro();
	}

	destruir(): void {
		cancelAnimationFrame(this.pedido);
		this.observador?.disconnect();
		for (const soltar of this.soltar) soltar();
		this.desenhista.destruir();
		this.documento = null;
	}

	/**
	 * Avisa as entidades do espaço que a tela não exibe (V6), uma vez por
	 * contagem: o mesmo aviso não se repete ao voltar ao espaço, e uma edição que
	 * muda a contagem avisa de novo.
	 */
	private avisarNaoExibidas(): void {
		const documento = this.documento;
		if (documento == null || this.aviso == null) return;

		const recusadas = documento.getScreenRefused(this.espaco);
		const total = recusadas.reduce((soma, { count }) => soma + count, 0);
		const mensagem =
			total === 0
				? ''
				: `A tela não exibe ${total} entidade(s) de ${this.espaco}: ` +
					recusadas.map(({ entityType, count }) => `${count} ${entityType}`).join(', ') +
					'.';

		if (this.avisados.get(this.espaco) === mensagem) return;
		this.avisados.set(this.espaco, mensagem);
		if (mensagem !== '') this.aviso(mensagem);
	}

	// -- blocos ----------------------------------------------------------------

	private readonly chaves = new Set<string>();

	private chave(layer: CadLayerId): string {
		return `${this.espaco}|${layer}`;
	}

	private traco(): Rgb {
		return this.escuro ? [255, 255, 255] : [0, 0, 0];
	}

	private coresDasCamadas(): Map<CadLayerId, Rgb> {
		const fundo = { escuro: this.escuro };
		return new Map(
			(this.documento?.listLayers() ?? []).map((c) => [c.id, corDaCamada(c.color, fundo)])
		);
	}

	private carregarCamadas(camadas: CadLayerId[]): void {
		const documento = this.documento;
		if (documento == null) return;

		const cores = this.coresDasCamadas();
		for (const layer of camadas) {
			const chave = this.chave(layer);
			this.desenhista.definirBloco(
				chave,
				documento.getScreenBlock(this.espaco, layer),
				cores.get(layer) ?? this.traco()
			);
			this.chaves.add(chave);
		}
		this.entidades = documento.getScreenEntityCount(this.espaco);
		this.caixa = this.caixaDoEspaco();
	}

	private caixaDoEspaco(): string {
		const caixa = this.documento?.getScreenExtents(this.espaco) ?? null;
		return caixa == null ? '' : `${caixa.minX} ${caixa.minY} ${caixa.maxX} ${caixa.maxY}`;
	}

	private limparBlocos(): void {
		for (const chave of this.chaves) this.desenhista.removerBloco(chave);
		this.chaves.clear();
		this.entidades = 0;
		this.caixa = '';
	}

	// -- quadro ----------------------------------------------------------------

	private pedirQuadro(): void {
		if (this.pedido !== 0) return;
		this.pedido = requestAnimationFrame(() => {
			this.pedido = 0;
			this.desenharQuadro();
		});
	}

	private desenharQuadro(): void {
		const transparente =
			this.aoMudarVista != null && this.espaco !== '' && this.espaco === this.espacos[0];
		const t = transformacao(this.vista, this.origem);
		this.desenhista.desenhar(
			t,
			this.canvas.width,
			this.canvas.height,
			transparente ? null : this.escuro ? FUNDO_ESCURO : FUNDO_CLARO
		);
		this.quadros += 1;

		const { centroX: x, centroY: y, escala } = this.vista;
		const dados = this.canvas.dataset;
		dados.quadros = String(this.quadros);
		dados.entidades = String(this.entidades);
		dados.espaco = this.espaco;
		dados.caixa = this.caixa;
		dados.vista = `${x} ${y} ${escala}`;
		dados.fundo = transparente ? 'transparente' : 'opaco';

		this.aoMudarVista?.(transparente ? { x, y, escala } : null);
	}

	private ajustarArea(): void {
		const largura = Math.max(1, this.canvas.clientWidth);
		const altura = Math.max(1, this.canvas.clientHeight);
		const razao = globalThis.devicePixelRatio || 1;
		this.canvas.width = Math.round(largura * razao);
		this.canvas.height = Math.round(altura * razao);

		// Antes de ter área, a vista enquadrada não vale: refaz ao ganhar tamanho.
		const semArea = this.vista.largura <= 1 || this.vista.altura <= 1;
		this.vista = redimensionar(this.vista, largura, altura);
		if (semArea && this.documento != null) {
			this.enquadrar();
		} else {
			this.pedirQuadro();
		}
	}

	// -- entrada ---------------------------------------------------------------

	private ouvir(tipo: string, ouvinte: (e: Event) => void, opcoes?: AddEventListenerOptions): void {
		this.canvas.addEventListener(tipo, ouvinte, opcoes);
		this.soltar.push(() => this.canvas.removeEventListener(tipo, ouvinte, opcoes));
	}

	private aoPressionar(e: PointerEvent): void {
		if (e.button !== BOTAO_DO_MEIO) return;
		e.preventDefault();
		this.arrasto = { x: e.clientX, y: e.clientY, id: e.pointerId };
		this.canvas.setPointerCapture?.(e.pointerId);
	}

	private aoMover(e: PointerEvent): void {
		if (this.arrasto == null || e.pointerId !== this.arrasto.id) return;
		this.vista = deslocar(this.vista, e.clientX - this.arrasto.x, e.clientY - this.arrasto.y);
		this.arrasto = { ...this.arrasto, x: e.clientX, y: e.clientY };
		this.pedirQuadro();
	}

	private aoSoltar(e: PointerEvent): void {
		if (this.arrasto == null || e.pointerId !== this.arrasto.id) return;
		this.arrasto = null;
		this.canvas.releasePointerCapture?.(e.pointerId);
	}

	private aoRolar(e: WheelEvent): void {
		e.preventDefault();
		const caixa = this.canvas.getBoundingClientRect();
		this.vista = ampliar(
			this.vista,
			fatorDaRoda(e.deltaY),
			e.clientX - caixa.left,
			e.clientY - caixa.top
		);
		this.pedirQuadro();
	}
}
