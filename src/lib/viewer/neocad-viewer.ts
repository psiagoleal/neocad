// Caminho relativo: src/lib/viewer/neocad-viewer.ts

/**
 * \file src/lib/viewer/neocad-viewer.ts
 * \brief Adaptador principal entre a UI Svelte do NeoCAD e o `cad-simple-viewer`.
 * \author Iago Leal
 * \date 2026-05-12
 */

import type {
	AcApDocManager as AcApDocManagerType,
	AcEdOpenMode as AcEdOpenModeType
} from '@mlightcad/cad-simple-viewer';
import type {
	CadDocumentPayload,
	CadDocumentSnapshot,
	CadLayout,
	CadOpenMode,
	CadViewerDocumentState,
	CadViewerProgressState
} from '$lib/types/cad';
import { buildDocumentSnapshot } from '$lib/services/cad-document';

const DEFAULT_CAD_DATA_BASE_URL = 'https://mlightcad.gitlab.io/cad-data/';
const DXF_PARSER_WORKER_URL = '/workers/dxf-parser-worker.js';
const LIBREDWG_PARSER_WORKER_URL = '/workers/libredwg-parser-worker.js';
const MTEXT_RENDERER_WORKER_URL = '/workers/mtext-renderer-worker.js';
const LIGHT_BACKGROUND = 0xf6f8fb;
const DARK_BACKGROUND = 0x081121;

type CadSimpleViewerModule = typeof import('@mlightcad/cad-simple-viewer');

type ViewerCallbacks = {
	onDocumentActivated?: (state: CadViewerDocumentState) => void;
	onProgress?: (progress: CadViewerProgressState) => void;
	onMessage?: (message: { kind: 'success' | 'warning' | 'info' | 'error'; text: string }) => void;
	onOpenRequested?: () => void | Promise<void>;
};

/** Nome com que o AutoCAD designa a aba do espaço-modelo. */
const MODEL_SPACE_TAB = 'Model';

/**
 * O que a montagem da lista de layouts precisa saber sobre o documento.
 *
 * A indireção existe para a lógica ser exercitável sem navegador: quem a testa
 * fornece objetos sintéticos no formato do upstream, e não um banco de dados
 * inteiro.
 */
export type CadLayoutSource = {
	/** Objetos `AcDbLayout` do dicionário de layouts. */
	layouts: Iterable<unknown>;
	/** Identificador do registro de bloco do espaço-modelo. */
	modelSpaceBlockId: string;
	/** Entidades de um registro de bloco, ou `null` se o registro não existir. */
	entityCountOf: (blockId: string) => number | null;
};

function readString(value: unknown): string | null {
	return typeof value === 'string' && value.length > 0 ? value : null;
}

function readNumber(value: unknown): number | null {
	return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

/**
 * Monta a lista de layouts do documento.
 *
 * # O espaço-modelo entra mesmo quando o dicionário não o traz
 *
 * Nem todo arquivo declara a aba `Model` no dicionário de layouts, e um desenho
 * sem ela ainda tem espaço-modelo. Sintetizar o item garante que a interface
 * sempre tenha para onde voltar — sem isso, abrir um arquivo assim deixaria o
 * usuário sem aba nenhuma que mostrasse o desenho.
 *
 * # A ordem é a das abas, e é determinística
 *
 * O espaço-modelo vem primeiro, como no AutoCAD; os demais por `tabOrder`, com
 * o nome como desempate para a lista não depender da ordem de iteração de um
 * dicionário.
 */
export function buildLayoutList(source: CadLayoutSource): CadLayout[] {
	const encontrados: CadLayout[] = [];
	let temModelSpace = false;

	for (const bruto of source.layouts) {
		if (typeof bruto !== 'object' || bruto === null) {
			continue;
		}

		const record = bruto as Record<string, unknown>;
		const blockId = readString(record.blockTableRecordId);
		const name = readString(record.layoutName);

		if (blockId == null || name == null) {
			continue;
		}

		const isModelSpace =
			blockId === source.modelSpaceBlockId || name.toLowerCase() === MODEL_SPACE_TAB.toLowerCase();
		temModelSpace = temModelSpace || isModelSpace;

		encontrados.push({
			name,
			blockId,
			tabOrder: readNumber(record.tabOrder) ?? 0,
			entityCount: source.entityCountOf(blockId) ?? 0,
			isModelSpace,
			viewportCount: Array.isArray(record.viewportArray) ? record.viewportArray.length : 0
		});
	}

	if (!temModelSpace) {
		encontrados.push({
			name: MODEL_SPACE_TAB,
			blockId: source.modelSpaceBlockId,
			tabOrder: 0,
			entityCount: source.entityCountOf(source.modelSpaceBlockId) ?? 0,
			isModelSpace: true,
			viewportCount: 0
		});
	}

	return encontrados.sort((um, outro) => {
		if (um.isModelSpace !== outro.isModelSpace) {
			return um.isModelSpace ? -1 : 1;
		}

		return um.tabOrder - outro.tabOrder || um.name.localeCompare(outro.name);
	});
}

/** O que a vista do upstream oferece para ser seguida. Existe para o teste simular. */
export type VistaDoUpstream = {
	readonly center: { x: number; y: number };
	flyTo(point: { x: number; y: number }, scale?: number): void;
	screenToWorld(point: { x: number; y: number }): { x: number; y: number };
};

/** Centro em coordenadas do desenho e escala em pixels por unidade. */
export type AlvoDaVista = { x: number; y: number; escala: number };

/** Pixels de tela usados para medir a escala da vista do upstream. */
const PASSO_DA_MEDIDA = 100;

/**
 * Entidades que o upstream ainda precisa desenhar: as que a tela do kernel não
 * desenha. Entidade sem tipo legível fica com o upstream, para não sumir.
 */
export function paraOUpstream<T>(entidades: T[], tiposDoKernel: ReadonlySet<string>): T[] {
	if (tiposDoKernel.size === 0) return entidades;

	return entidades.filter((entidade) => {
		const tipo = (entidade as { dxfTypeName?: unknown } | null)?.dxfTypeName;
		return typeof tipo !== 'string' || !tiposDoKernel.has(tipo.toUpperCase());
	});
}

/** Unidades do desenho que cabem em `PASSO_DA_MEDIDA` pixels, na vista atual. */
function unidadesNoPasso(vista: VistaDoUpstream): number {
	const a = vista.screenToWorld({ x: 0, y: 0 });
	const b = vista.screenToWorld({ x: PASSO_DA_MEDIDA, y: 0 });
	return Math.abs(b.x - a.x);
}

/**
 * Põe a vista do upstream no centro e na escala da tela do kernel.
 *
 * A câmera do upstream não fala em pixels: a escala dela depende da altura da
 * tela. Por isso o zoom 1 é medido antes, e o zoom final sai da proporção.
 */
export function seguirVista(vista: VistaDoUpstream, alvo: AlvoDaVista): void {
	if (!(Number.isFinite(alvo.escala) && alvo.escala > 0)) return;
	if (!(Number.isFinite(alvo.x) && Number.isFinite(alvo.y))) return;

	vista.flyTo({ x: alvo.x, y: alvo.y }, 1);
	const porUnidadeNoZoom1 = PASSO_DA_MEDIDA / unidadesNoPasso(vista);
	if (!(Number.isFinite(porUnidadeNoZoom1) && porUnidadeNoZoom1 > 0)) return;

	vista.flyTo({ x: alvo.x, y: alvo.y }, alvo.escala / porUnidadeNoZoom1);
}

/** Centro e escala, em pixels por unidade, que a vista do upstream mostra agora. */
export function medirVista(vista: VistaDoUpstream): AlvoDaVista {
	const { x, y } = vista.center;
	return { x, y, escala: PASSO_DA_MEDIDA / unidadesNoPasso(vista) };
}

/**
 * Tira do upstream todo caminho de comando: a barra dele e o \`sendStringToExecute\`,
 * por onde passam a barra e a tecla Delete solta no documento. Quem edita é o
 * kernel, pela linha de comando própria (spec 07, V5).
 */
export function desligarComandosDoUpstream(
	docManager: { sendStringToExecute(texto: string): void },
	configuracao: { isShowCommandLine: boolean }
): void {
	configuracao.isShowCommandLine = false;
	docManager.sendStringToExecute = () => undefined;
}

/** Vista do upstream, só no que o fundo precisa. */
export type VistaComFundo = { renderer: unknown; backgroundColor: number };

/**
 * Põe o fundo do tema no upstream, que é o que aparece sob a tela própria.
 *
 * O upstream só desenha a cor 7 em branco quando o fundo é exatamente preto: o
 * material de cada entidade nova escolhe a cor pelo \`currentBackgroundColor\` do
 * renderizador. Com o azul-escuro do tema, ela sairia preta. Por isso, no escuro,
 * a cor de limpeza é a do tema, o renderizador passa a contar o fundo como preto
 * e as entidades já desenhadas são recoloridas. Sem esse renderizador, o escuro
 * cai no preto puro, que mantém a cor 7 legível.
 */
export function aplicarFundoDoUpstream(vista: VistaComFundo, escuro: boolean): void {
	const renderer = vista.renderer as {
		currentBackgroundColor?: number;
		changeForeground?: (cor: number) => void;
	} | null;

	if (!escuro) {
		vista.backgroundColor = LIGHT_BACKGROUND;
		return;
	}
	if (
		renderer == null ||
		typeof renderer.changeForeground !== 'function' ||
		typeof renderer.currentBackgroundColor !== 'number'
	) {
		vista.backgroundColor = 0;
		return;
	}

	vista.backgroundColor = DARK_BACKGROUND;
	renderer.currentBackgroundColor = 0;
	renderer.changeForeground(0xffffff);
}

export class NeoCadViewer {
	private cadModule: CadSimpleViewerModule | null = null;
	private docManager: AcApDocManagerType | null = null;
	private activeDocumentSource: CadDocumentPayload['source'] | undefined;
	private activeDocumentPath: string | undefined;
	private readonly callbacks: ViewerCallbacks;
	private readonly eventHandlers: {
		onOpenRequested: () => void;
		onProgress: (payload: {
			percentage: number;
			stage: string;
			subStage?: string;
			subStageStatus?: string;
		}) => void;
		onMessage: (payload: {
			message: string;
			type: 'success' | 'warning' | 'info' | 'error';
		}) => void;
		onFailedToOpenFile: (payload: { fileName: string }) => void;
		onMissingFont: (payload: { fontName: string; count: number }) => void;
	};
	/** Contêiner do upstream; os atributos dele são o que os testes observam (spec 07). */
	private host: HTMLElement | null = null;
	/** Tipos que a tela do kernel desenha na próxima abertura; vazio = o upstream desenha tudo. */
	private tiposDoKernel: ReadonlySet<string> = new Set();
	/** Vista da tela do kernel que o upstream segue; `null` quando não há o que seguir. */
	private alvo: AlvoDaVista | null = null;
	private aplicandoVista = false;
	/** Tema do fundo do upstream. */
	private escuro = true;
	private readonly aoMudarVistaDoUpstream = () => this.reaplicarVista();
	private documentActivatedHandler?: (payload: {
		doc: { fileName: string; docTitle: string };
		mode: AcEdOpenModeType;
	}) => void;

	constructor(callbacks: ViewerCallbacks = {}) {
		this.callbacks = callbacks;
		this.eventHandlers = {
			onOpenRequested: () => {
				void this.callbacks.onOpenRequested?.();
			},
			onProgress: (payload) => {
				this.callbacks.onProgress?.({
					percentage: payload.percentage,
					stage: payload.stage,
					subStage: payload.subStage,
					subStageStatus: payload.subStageStatus
				});
			},
			onMessage: (payload) => {
				this.callbacks.onMessage?.({ kind: payload.type, text: payload.message });
			},
			onFailedToOpenFile: (payload) => {
				this.callbacks.onMessage?.({
					kind: 'error',
					text: `Não foi possível abrir o arquivo ${payload.fileName}.`
				});
			},
			onMissingFont: (payload) => {
				this.callbacks.onMessage?.({
					kind: 'warning',
					text: `Fonte não encontrada: ${payload.fontName} (${payload.count} entidades).`
				});
			}
		};
	}

	private async ensureCadModule(): Promise<CadSimpleViewerModule> {
		this.cadModule ??= await import('@mlightcad/cad-simple-viewer');
		return this.cadModule;
	}

	private resolveOpenMode(module: CadSimpleViewerModule, mode: CadOpenMode): AcEdOpenModeType {
		switch (mode) {
			case 'read':
				return module.AcEdOpenMode.Read;
			case 'review':
				return module.AcEdOpenMode.Review;
			default:
				return module.AcEdOpenMode.Write;
		}
	}

	async mount(container: HTMLElement): Promise<void> {
		await this.destroy();
		container.innerHTML = '';

		const cadModule = await this.ensureCadModule();

		this.docManager =
			cadModule.AcApDocManager.createInstance({
				container,
				autoResize: true,
				baseUrl: DEFAULT_CAD_DATA_BASE_URL,
				useMainThreadDraw: false,
				webworkerFileUrls: {
					dxfParser: DXF_PARSER_WORKER_URL,
					dwgParser: LIBREDWG_PARSER_WORKER_URL,
					mtextRender: MTEXT_RENDERER_WORKER_URL
				}
			}) ?? null;

		if (this.docManager == null) {
			throw new Error('Falha ao inicializar o document manager do CAD viewer.');
		}

		this.documentActivatedHandler = ({ doc, mode }) => {
			this.callbacks.onDocumentActivated?.({
				fileName: doc.fileName,
				docTitle: doc.docTitle,
				mode:
					mode === cadModule.AcEdOpenMode.Read
						? 'read'
						: mode === cadModule.AcEdOpenMode.Review
							? 'review'
							: 'write',
				source: this.activeDocumentSource,
				path: this.activeDocumentPath
			});
		};

		this.host = container;
		desligarComandosDoUpstream(this.docManager, cadModule.AcApSettingManager.instance);
		// O tema começa escuro (a página e a tela do kernel também); como a tela do
		// kernel é transparente, é o fundo do upstream que aparece.
		aplicarFundoDoUpstream(this.docManager.curView, true);
		this.escuro = true;
		this.docManager.curView.events.viewChanged.addEventListener(this.aoMudarVistaDoUpstream);
		this.docManager.curView.events.viewResize.addEventListener(this.aoMudarVistaDoUpstream);
		this.docManager.events.documentActivated.addEventListener(this.documentActivatedHandler);
		cadModule.eventBus.on('open-file', this.eventHandlers.onOpenRequested);
		cadModule.eventBus.on('open-file-progress', this.eventHandlers.onProgress);
		cadModule.eventBus.on('message', this.eventHandlers.onMessage);
		cadModule.eventBus.on('failed-to-open-file', this.eventHandlers.onFailedToOpenFile);
		cadModule.eventBus.on('font-not-found', this.eventHandlers.onMissingFont);
	}

	async openDocument(payload: CadDocumentPayload, mode: CadOpenMode = 'write'): Promise<boolean> {
		if (this.docManager == null) {
			throw new Error('Viewer CAD ainda não foi inicializado.');
		}

		const cadModule = await this.ensureCadModule();
		this.activeDocumentSource = payload.source;
		this.activeDocumentPath = payload.path;

		// Durante a leitura, o que a tela do kernel desenha não chega à cena do
		// upstream. Depois dela, nada mais entra na cena: o upstream não executa
		// comando algum (MT-KV-13b).
		const vista = this.docManager.curView;
		const original = vista.addEntity;
		const tipos = this.tiposDoKernel;
		let entregues = 0;
		vista.addEntity = (entidade) => {
			const restantes = paraOUpstream(Array.isArray(entidade) ? entidade : [entidade], tipos);
			entregues += restantes.length;
			if (restantes.length > 0) original.call(vista, restantes);
		};

		try {
			return await this.docManager.openDocument(payload.fileName, payload.content, {
				mode: this.resolveOpenMode(cadModule, mode)
			});
		} finally {
			vista.addEntity = original;
			if (this.host != null) this.host.dataset.entidades = String(entregues);
		}
	}

	/**
	 * Tipos de entidade que a tela do kernel desenha (`CadDocument.getScreenEntityTypes`).
	 * Valem para a próxima abertura; lista vazia faz o upstream desenhar tudo.
	 */
	definirTiposDoKernel(tipos: Iterable<string>): void {
		this.tiposDoKernel = new Set([...tipos].map((tipo) => tipo.toUpperCase()));
	}

	/**
	 * Faz a vista do upstream seguir a da tela do kernel; `null` deixa de seguir.
	 * A vista medida depois vai para `data-vista` do contêiner (spec 07).
	 */
	seguirVistaDoKernel(alvo: AlvoDaVista | null): void {
		this.alvo = alvo;
		this.reaplicarVista();
	}

	private reaplicarVista(): void {
		const docManager = this.docManager;
		if (docManager == null || this.aplicandoVista) return;

		const vista = docManager.curView as unknown as VistaDoUpstream;
		this.aplicandoVista = true;
		try {
			if (this.alvo != null) {
				seguirVista(vista, this.alvo);
				docManager.curView.isDirty = true;
			}
		} finally {
			this.aplicandoVista = false;
		}

		if (this.host != null) {
			const medida = this.alvo == null ? null : medirVista(vista);
			this.host.dataset.vista = medida == null ? '' : `${medida.x} ${medida.y} ${medida.escala}`;
		}
	}

	zoomToFit(): void {
		this.docManager?.curView.zoomToFitDrawing();
	}

	toggleBackground(): 'light' | 'dark' {
		if (this.docManager == null) {
			return 'dark';
		}

		this.escuro = !this.escuro;
		aplicarFundoDoUpstream(this.docManager.curView, this.escuro);
		return this.escuro ? 'dark' : 'light';
	}

	/**
	 * Extrai do documento aberto pelo upstream o retrato que alimenta o kernel.
	 *
	 * Enquanto K5 e K6 não chegam, o upstream continua sendo quem lê o arquivo e
	 * quem desenha; o kernel passa a ser a fonte de verdade sobre **o que existe**
	 * no desenho. As duas representações convivem durante a transição, e é por
	 * isso que a contagem de entidades de uma e de outra precisa bater.
	 *
	 * Devolve `null` quando não há documento ativo.
	 */
	extractDocumentSnapshot(): CadDocumentSnapshot | null {
		const database = this.docManager?.curDocument?.database;

		if (database == null) {
			return null;
		}

		const layerTable = database.tables.layerTable;
		const modelSpace = database.tables.blockTable.modelSpace;

		return buildDocumentSnapshot(layerTable.newIterator(), modelSpace.newIterator());
	}

	/**
	 * Lista os layouts do documento aberto, com o espaço-modelo entre eles.
	 *
	 * Devolve lista vazia quando não há documento ativo. A contagem de entidades
	 * de cada layout vem do registro de bloco associado, que é como o formato —
	 * e o ADR 0005 — modelam layout.
	 */
	listLayouts(): CadLayout[] {
		const database = this.docManager?.curDocument?.database;

		if (database == null) {
			return [];
		}

		const blockTable = database.tables.blockTable;

		return buildLayoutList({
			layouts: database.objects.layout.entries(),
			modelSpaceBlockId: blockTable.modelSpace.objectId,
			entityCountOf: (blockId) => {
				const record = blockTable.getIdAt(blockId);

				return record == null ? null : record.newIterator().count;
			}
		});
	}

	async destroy(): Promise<void> {
		if (this.cadModule) {
			this.cadModule.eventBus.off('open-file', this.eventHandlers.onOpenRequested);
			this.cadModule.eventBus.off('open-file-progress', this.eventHandlers.onProgress);
			this.cadModule.eventBus.off('message', this.eventHandlers.onMessage);
			this.cadModule.eventBus.off('failed-to-open-file', this.eventHandlers.onFailedToOpenFile);
			this.cadModule.eventBus.off('font-not-found', this.eventHandlers.onMissingFont);
		}

		if (this.docManager) {
			this.docManager.curView.events.viewChanged.removeEventListener(this.aoMudarVistaDoUpstream);
			this.docManager.curView.events.viewResize.removeEventListener(this.aoMudarVistaDoUpstream);
		}

		if (this.docManager && this.documentActivatedHandler) {
			this.docManager.events.documentActivated.removeEventListener(this.documentActivatedHandler);
		}

		if (this.cadModule) {
			try {
				await this.cadModule.AcApDocManager.instance.destroy();
			} catch {
				// Nenhuma instância ativa para destruir.
			}
		}

		this.docManager = null;
		this.host = null;
		this.alvo = null;
		this.activeDocumentSource = undefined;
		this.activeDocumentPath = undefined;
		this.documentActivatedHandler = undefined;
	}
}
