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
import type { CadEntityId, CadLayerId, CadPoint, CadSnap, CadSnapMode } from '$lib/types/cad';
import { DesenhistaCanvas2d } from './canvas2d';
import { corDaCamada, type Rgb } from './cor';
import type { Desenhista } from './desenhista';
import type { Escolha } from '$lib/command-line/motor';
import { Selecao } from './selecao';
import { DesenhistaWebgl2 } from './webgl2';
import {
	ampliar,
	deslocar,
	enquadrar,
	erroDeCorda,
	fatorDaRoda,
	paraMundo,
	redimensionar,
	transformacao,
	type Vista
} from './vista';

const FUNDO_ESCURO: Rgb = [8, 17, 33];
const FUNDO_CLARO: Rgb = [246, 248, 251];

/** Botão do meio do mouse, o do pan (spec 07). */
const BOTAO_DO_MEIO = 1;
const BOTAO_ESQUERDO = 0;

/** Distância, em pixels, até onde o clique pega um traço (spec 10, S1). */
const RAIO_DO_CLIQUE_PX = 5;

/** Blocos da seleção: desenhados por último, por cima das camadas. */
const BLOCO_DO_DESTAQUE = '__destaque';
const BLOCO_DA_JANELA = '__janela';
const COR_DO_DESTAQUE: Rgb = [90, 170, 255];
const COR_DA_JANELA: Rgb = [90, 140, 255];
const COR_DA_CRUZADA: Rgb = [90, 220, 130];

/** Distância, em pixels, até onde o cursor prende a um ponto notável (spec 10, P2). */
const RAIO_DO_PRENDIMENTO_PX = 10;
const MODOS_DO_PRENDIMENTO: CadSnapMode[] = [
	'endpoint',
	'midpoint',
	'center',
	'quadrant',
	'intersection'
];
const BLOCO_DO_MARCADOR = '__marcador';
const COR_DO_MARCADOR: Rgb = [255, 210, 60];
/** Meio lado do quadrado do marcador, em pixels. */
const MARCADOR_PX = 5;

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
	private readonly selecao = new Selecao();
	/** Com o modo de escolha ligado, o clique vai ao comando (spec 10, T4). */
	private aoEscolher: ((escolha: Escolha) => void) | null = null;
	/** Com o modo de ponto ligado, o clique entrega o ponto ao comando (spec 10, P1). */
	private aoApontar: ((ponto: CadPoint) => void) | null = null;
	private consultaNaoModelado: ((ponto: CadPoint, raioPx: number) => string | null) | null = null;
	private avisoNaoEditavel: ((tipo: string) => void) | null = null;
	/** O ponto notável sob o cursor, no modo de ponto. */
	private prendimento: CadSnap | null = null;
	/** Onde está o cursor, em coordenadas do desenho, enquanto a janela está aberta. */
	private cursor: { x: number; y: number } | null = null;
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
		// Esc fora de um campo de texto esvazia a seleção (spec 10, S5); dentro da
		// linha de comando, quem decide é ela.
		if (typeof document !== 'undefined') {
			const aoTeclar = (e: KeyboardEvent) => {
				const alvo = e.target as HTMLElement | null;
				const emCampo = alvo?.tagName === 'INPUT' || alvo?.tagName === 'TEXTAREA';
				if (e.key === 'Escape' && !emCampo) this.esvaziarSelecao();
			};
			document.addEventListener('keydown', aoTeclar);
			this.soltar.push(() => document.removeEventListener('keydown', aoTeclar));
		}

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
		this.selecao.esvaziar();
		this.cursor = null;
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
		// A apagada sai da seleção; a movida leva o destaque junto (spec 10, S4).
		const ids = this.selecao.ids;
		if (ids.length > 0) {
			const existentes = documento.getEntitiesById(ids);
			this.selecao.podar((id) => existentes[ids.indexOf(id)] != null);
		}
		this.atualizarDestaque();
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

	/**
	 * Põe a tela no modo de escolha: o clique esquerdo vai ao comando, com o que
	 * está sob ele, o ponto e o raio, e não mexe na seleção (spec 10, T4); `null`
	 * volta ao modo de seleção.
	 */
	definirEscolha(aoEscolher: ((escolha: Escolha) => void) | null): void {
		this.aoEscolher = aoEscolher;
		if (aoEscolher != null) {
			this.aoApontar = null;
			this.prendimento = null;
			this.atualizarMarcador();
			this.fecharJanela();
		}
		this.pedirQuadro();
	}

	/** Uma janela aberta antes não continua nos modos de escolha e de ponto. */
	private fecharJanela(): void {
		this.selecao.esvaziarJanela();
		this.cursor = null;
		this.atualizarJanela();
	}

	/**
	 * Põe a tela no modo de ponto: o cursor prende aos pontos notáveis (com
	 * marcador) e o clique entrega o ponto ao comando, sem mexer na seleção
	 * (spec 10, P1, P2); `null` volta ao modo de seleção.
	 */
	definirPonto(aoApontar: ((ponto: CadPoint) => void) | null): void {
		this.aoApontar = aoApontar;
		if (aoApontar != null) {
			this.aoEscolher = null;
			this.fecharJanela();
		}
		this.prendimento = null;
		this.atualizarMarcador();
		this.pedirQuadro();
	}

	/**
	 * Quem diz o tipo da entidade que só o upstream desenha sob um ponto (spec 07,
	 * V7); o raio vai em pixels.
	 */
	consultarNaoModelado(
		consulta: ((ponto: CadPoint, raioPx: number) => string | null) | null
	): void {
		this.consultaNaoModelado = consulta;
	}

	/** Para onde vai o aviso do clique em entidade não editável, no modo de seleção (V7). */
	aoNaoEditavel(aviso: ((tipo: string) => void) | null): void {
		this.avisoNaoEditavel = aviso;
	}

	/** Todas as entidades desenhadas no espaço mostrado. */
	todas(): CadEntityId[] {
		const documento = this.documento;
		const caixa = documento?.getScreenExtents(this.espaco) ?? null;
		if (documento == null || caixa == null) return [];
		return documento.selectOnScreen(this.espaco, caixa, true);
	}

	/** Para onde vai o aviso das entidades que a tela não exibe (spec 07, V6). */
	avisarCom(aviso: (mensagem: string) => void): void {
		this.aviso = aviso;
	}

	/** Esvazia a seleção e fecha a janela aberta (spec 10, S5). */
	esvaziarSelecao(): void {
		this.selecao.esvaziar();
		this.cursor = null;
		this.atualizarDestaque();
		this.atualizarJanela();
		this.pedirQuadro();
	}

	/** Ids selecionados no espaço mostrado. */
	get selecionados(): CadEntityId[] {
		return this.selecao.ids;
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
		this.desenhista.removerBloco(BLOCO_DO_DESTAQUE);
		this.desenhista.removerBloco(BLOCO_DA_JANELA);
		this.desenhista.removerBloco(BLOCO_DO_MARCADOR);
		this.entidades = 0;
		this.caixa = '';
	}

	// -- seleção ---------------------------------------------------------------

	/** O destaque vem do kernel; é refeito por último, para ficar por cima. */
	private atualizarDestaque(): void {
		this.desenhista.removerBloco(BLOCO_DO_DESTAQUE);
		const ids = this.selecao.ids;
		if (this.documento == null || ids.length === 0) return;
		this.desenhista.definirBloco(
			BLOCO_DO_DESTAQUE,
			this.documento.getScreenSegmentsOf(this.espaco, ids),
			COR_DO_DESTAQUE
		);
	}

	/** O retângulo da janela aberta, do início ao cursor, relativo à origem. */
	private atualizarJanela(): void {
		this.desenhista.removerBloco(BLOCO_DA_JANELA);
		const inicio = this.selecao.inicioDaJanela;
		const cursor = this.cursor;
		if (inicio == null || cursor == null) return;

		const { x: ox, y: oy } = this.origem;
		const [x0, y0, x1, y1] = [inicio.x - ox, inicio.y - oy, cursor.x - ox, cursor.y - oy];
		this.desenhista.definirBloco(
			BLOCO_DA_JANELA,
			new Float32Array([x0, y0, x1, y0, x1, y0, x1, y1, x1, y1, x0, y1, x0, y1, x0, y0]),
			cursor.x < inicio.x ? COR_DA_CRUZADA : COR_DA_JANELA
		);
	}

	/** O quadrado em volta do ponto notável sob o cursor, por cima de tudo. */
	private atualizarMarcador(): void {
		this.desenhista.removerBloco(BLOCO_DO_MARCADOR);
		const achado = this.prendimento;
		if (achado == null) return;
		const m = MARCADOR_PX / this.vista.escala;
		const [x, y] = [achado.point.x - this.origem.x, achado.point.y - this.origem.y];
		this.desenhista.definirBloco(
			BLOCO_DO_MARCADOR,
			new Float32Array([
				x - m,
				y - m,
				x + m,
				y - m,
				x + m,
				y - m,
				x + m,
				y + m,
				x + m,
				y + m,
				x - m,
				y + m,
				x - m,
				y + m,
				x - m,
				y - m
			]),
			COR_DO_MARCADOR
		);
	}

	/** O ponto notável ao alcance do ponto, no espaço mostrado. */
	private prender(ponto: CadPoint): CadSnap | null {
		const documento = this.documento;
		if (documento == null || this.espaco === '') return null;
		return documento.snapOnScreen(
			this.espaco,
			ponto,
			RAIO_DO_PRENDIMENTO_PX / this.vista.escala,
			MODOS_DO_PRENDIMENTO
		);
	}

	private pontoDoDesenho(e: PointerEvent): { x: number; y: number } {
		const caixa = this.canvas.getBoundingClientRect();
		return paraMundo(this.vista, e.clientX - caixa.left, e.clientY - caixa.top);
	}

	private aoClicar(e: PointerEvent): void {
		const documento = this.documento;
		if (documento == null || this.espaco === '') return;
		e.preventDefault();

		const espaco = this.espaco;
		const ponto = this.pontoDoDesenho(e);
		const raio = RAIO_DO_CLIQUE_PX / this.vista.escala;
		if (this.aoApontar != null) {
			this.aoApontar(this.prender(ponto)?.point ?? ponto);
			return;
		}
		const ids = documento.pickOnScreen(espaco, ponto, raio);
		// Nada do kernel sob o clique: talvez uma entidade que só o upstream desenha (V7).
		// Só vale com o upstream por baixo, no espaço-modelo; num layout ele não está ali.
		const sobreposta = this.aoMudarVista != null && this.espaco === this.espacos[0];
		const naoEditavel =
			ids.length === 0 && sobreposta
				? (this.consultaNaoModelado?.(ponto, RAIO_DO_CLIQUE_PX) ?? null)
				: null;
		if (this.aoEscolher != null) {
			this.aoEscolher(
				naoEditavel == null ? { ids, ponto, raio } : { ids, ponto, raio, naoEditavel }
			);
			return;
		}
		if (naoEditavel != null && this.selecao.inicioDaJanela == null) {
			this.avisoNaoEditavel?.(naoEditavel);
			return;
		}
		this.selecao.clicar(ponto, raio, e.shiftKey, {
			clique: (p, raio) => documento.pickOnScreen(espaco, p, raio),
			area: (area, cruzada) => documento.selectOnScreen(espaco, area, cruzada)
		});
		this.cursor = this.selecao.inicioDaJanela == null ? null : ponto;
		this.atualizarDestaque();
		this.atualizarJanela();
		this.pedirQuadro();
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
		dados.selecao = String(this.selecao.ids.length);
		dados.clique =
			this.aoApontar != null ? 'ponto' : this.aoEscolher != null ? 'escolha' : 'selecao';
		const achado = this.prendimento;
		dados.prendimento = achado == null ? '' : `${achado.mode} ${achado.point.x} ${achado.point.y}`;

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
		if (e.button === BOTAO_ESQUERDO) {
			this.aoClicar(e);
			return;
		}
		if (e.button !== BOTAO_DO_MEIO) return;
		e.preventDefault();
		this.arrasto = { x: e.clientX, y: e.clientY, id: e.pointerId };
		this.canvas.setPointerCapture?.(e.pointerId);
	}

	private aoMover(e: PointerEvent): void {
		if (this.aoApontar != null) {
			this.prendimento = this.prender(this.pontoDoDesenho(e));
			this.atualizarMarcador();
			this.pedirQuadro();
		}
		if (this.selecao.inicioDaJanela != null) {
			this.cursor = this.pontoDoDesenho(e);
			this.atualizarJanela();
			this.pedirQuadro();
		}
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
