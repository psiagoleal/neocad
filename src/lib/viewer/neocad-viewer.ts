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
	CadOpenMode,
	CadViewerDocumentState,
	CadViewerProgressState
} from '$lib/types/cad';

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

		return this.docManager.openDocument(payload.fileName, payload.content, {
			mode: this.resolveOpenMode(cadModule, mode)
		});
	}

	zoomToFit(): void {
		this.docManager?.curView.zoomToFitDrawing();
	}

	toggleBackground(): 'light' | 'dark' {
		if (this.docManager == null) {
			return 'dark';
		}

		const nextColor =
			this.docManager.curView.backgroundColor === DARK_BACKGROUND
				? LIGHT_BACKGROUND
				: DARK_BACKGROUND;
		this.docManager.curView.backgroundColor = nextColor;
		return nextColor === LIGHT_BACKGROUND ? 'light' : 'dark';
	}

	executeCommand(command: string): void {
		if (this.docManager == null) {
			throw new Error('Viewer CAD ainda não foi inicializado.');
		}

		this.docManager.sendStringToExecute(command);
	}

	async destroy(): Promise<void> {
		if (this.cadModule) {
			this.cadModule.eventBus.off('open-file', this.eventHandlers.onOpenRequested);
			this.cadModule.eventBus.off('open-file-progress', this.eventHandlers.onProgress);
			this.cadModule.eventBus.off('message', this.eventHandlers.onMessage);
			this.cadModule.eventBus.off('failed-to-open-file', this.eventHandlers.onFailedToOpenFile);
			this.cadModule.eventBus.off('font-not-found', this.eventHandlers.onMissingFont);
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
		this.activeDocumentSource = undefined;
		this.activeDocumentPath = undefined;
		this.documentActivatedHandler = undefined;
	}
}
