// Caminho relativo: src/lib/types/cad.ts

/**
 * \file src/lib/types/cad.ts
 * \brief Tipos compartilhados para abertura de documentos CAD e estado do viewer.
 * \author Iago Leal
 * \date 2026-05-12
 */

export type CadDocumentSource = 'tauri' | 'browser';
export type CadOpenMode = 'read' | 'review' | 'write';
export type ViewerMessageKind = 'success' | 'warning' | 'info' | 'error';

export interface CadDocumentPayload {
	fileName: string;
	content: ArrayBuffer;
	source: CadDocumentSource;
	path?: string;
}

export interface CadViewerDocumentState {
	fileName: string;
	docTitle: string;
	mode: CadOpenMode;
	source?: CadDocumentSource;
	path?: string;
}

export interface CadViewerProgressState {
	percentage: number;
	stage: string;
	subStage?: string;
	subStageStatus?: string;
}

export interface CadViewerMessage {
	id: string;
	kind: ViewerMessageKind;
	text: string;
}
