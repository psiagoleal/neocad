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

export interface CadRecentDocument {
	fileName: string;
	path?: string;
	source: CadDocumentSource;
	openedAt: string;
}

/** Categorias de apresentação para os comandos CAD no catálogo do menu `Ajuda`. */
export type CadCommandCategory = 'navigation' | 'draw' | 'modify' | 'layer' | 'system' | 'other';

/**
 * Descritor cru de um comando, lido do command stack do upstream em tempo de
 * execução. É a fonte de verdade sobre quais comandos existem (ver ADR 0001).
 */
export interface CadCommandDescriptor {
	/** Nome global (untranslated) — string enviada ao viewer. */
	globalName: string;
	/** Nome local (traduzido pelo upstream), quando disponível. */
	localName: string;
	/** Grupo de origem no upstream (ex.: ACAD, USER). */
	group: string;
}

/**
 * Item do catálogo de comandos exibido na UI: une o descritor de runtime aos
 * metadados de apresentação em PT-BR.
 */
export interface CadCommandCatalogItem {
	/** Identificador estável (nome global normalizado em maiúsculas). */
	id: string;
	/** Nome global enviado ao viewer via `executeCommand`. */
	command: string;
	/** Rótulo amigável em PT-BR. */
	label: string;
	/** Categoria de apresentação. */
	category: CadCommandCategory;
	/** Grupo de origem no upstream. */
	group: string;
	/** Observação opcional de uso. */
	notes?: string;
}
