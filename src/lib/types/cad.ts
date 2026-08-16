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

// -- Contratos do documento do kernel ----------------------------------------
//
// Estes tipos são a forma NeoCAD do documento. A UI depende deles, nunca das
// estruturas do kernel WebAssembly nem do upstream (ADR 0001 e ADR 0003): a
// tradução acontece uma vez, em `src/lib/services/cad-document.ts`.

/**
 * Identificador opaco de camada.
 *
 * O `brand` existe para o compilador recusar a troca entre identificador de
 * camada e de entidade. O kernel distingue os dois em tipos; perder essa
 * distinção ao cruzar a ponte seria um retrocesso — do lado JavaScript ambos são
 * apenas texto.
 */
export type CadLayerId = string & { readonly brand: unique symbol };

/** Identificador opaco de entidade. Ver [`CadLayerId`] sobre o `brand`. */
export type CadEntityId = string & { readonly brand: unique symbol };

/** Ponto no plano, em unidades do desenho. */
export interface CadPoint {
	x: number;
	y: number;
}

/** Caixa envolvente alinhada aos eixos. */
export interface CadBounds {
	minX: number;
	minY: number;
	maxX: number;
	maxY: number;
}

/**
 * Cor: índice na paleta ACI ou cor verdadeira.
 *
 * Os extremos da paleta não são cores: o índice `0` significa "herda do bloco" e
 * o `256`, "herda da camada". Ficam como variantes próprias para que nenhum
 * consumidor precise lembrar da convenção.
 */
export type CadColor =
	| { kind: 'byBlock' }
	| { kind: 'byLayer' }
	| { kind: 'index'; index: number }
	| { kind: 'rgb'; red: number; green: number; blue: number };

/** Camada do documento. */
export interface CadLayer {
	id: CadLayerId;
	name: string;
	color: CadColor;
	/** Falso quando a camada está desligada **ou** congelada. */
	isVisible: boolean;
	isOff: boolean;
	isFrozen: boolean;
	isLocked: boolean;
}

/** Forma geométrica de uma entidade. */
export type CadGeometry =
	| { kind: 'line'; start: CadPoint; end: CadPoint }
	| { kind: 'circle'; center: CadPoint; radius: number }
	| { kind: 'arc'; center: CadPoint; radius: number; startAngle: number; endAngle: number }
	| { kind: 'polyline'; vertices: CadPoint[]; closed: boolean }
	| { kind: 'text'; position: CadPoint; content: string; height: number; rotation: number };

/** Entidade de desenho. */
export interface CadEntity {
	id: CadEntityId;
	layerId: CadLayerId;
	geometry: CadGeometry;
	bounds: CadBounds;
}

/**
 * Estado da pilha de comandos, na forma que o menu `Editar` consome.
 *
 * Os rótulos vêm nomeados como ação — `Desenhar linha`, `Apagar` — para a UI
 * compor `Desfazer <rótulo>` sem precisar conhecer os comandos.
 */
export interface CadHistoryState {
	canUndo: boolean;
	canRedo: boolean;
	undoLabel: string | null;
	redoLabel: string | null;
	undoDepth: number;
	redoDepth: number;
}

/** Camada de um documento a carregar no kernel. Referenciada por nome. */
export interface CadLayerSnapshot {
	name: string;
	color: CadColor;
	isOff: boolean;
	isFrozen: boolean;
	isLocked: boolean;
}

/** Entidade de um documento a carregar no kernel. */
export interface CadEntitySnapshot {
	/** Nome da camada — o parser não conhece os identificadores do kernel. */
	layerName: string;
	geometry: CadGeometry;
}

/**
 * Entidade que o kernel ainda não modela.
 *
 * Registrada em vez de descartada em silêncio: é a medida de quanto de um
 * arquivo real o kernel ainda não cobre, e a lista que orienta o que implementar
 * em seguida.
 */
export interface CadUnsupportedEntity {
	/** Tipo relatado pelo upstream, quando disponível. */
	type: string;
	layerName: string;
}

/** Documento extraído do upstream, pronto para carregar no kernel. */
export interface CadDocumentSnapshot {
	layers: CadLayerSnapshot[];
	entities: CadEntitySnapshot[];
	unsupported: CadUnsupportedEntity[];
}

/** Resumo do que o kernel recebeu ao carregar um documento. */
export interface CadLoadReport {
	layerCount: number;
	entityCount: number;
	/** Entidades recusadas pelo kernel por referenciarem camada inexistente. */
	skippedCount: number;
	/** Entidades que a extração não soube converter. */
	unsupportedCount: number;
}

/** Um tipo de entidade que o kernel ainda não representa, com sua contagem. */
export interface CadUnsupportedCount {
	entityType: string;
	count: number;
}

/** Um layout de espaço-papel encontrado no arquivo. */
export interface CadPaperSpaceLayout {
	name: string;
	entityCount: number;
}

/**
 * O que uma gravação descartaria do desenho aberto.
 *
 * Existe para a perda **aparecer antes de acontecer**. Salvar por cima de um
 * original sem avisar que a prancha ou as cotas não vão junto é destruição
 * silenciosa de trabalho alheio; o ADR 0005 proíbe, e este contrato é o que
 * permite cumprir a proibição na interface.
 */
export interface CadSaveLoss {
	unsupported: CadUnsupportedCount[];
	unsupportedCount: number;
	paperSpace: CadPaperSpaceLayout[];
	paperSpaceCount: number;
	xrefCount: number;
	isLossless: boolean;
}

/**
 * Resultado da abertura de um DXF pela leitura nativa do kernel.
 *
 * Substitui, para DXF, o `CadLoadReport` do retrato extraído do upstream: aqui
 * o kernel leu o arquivo por conta própria, então sabe mais — quantos blocos
 * há, o que não compreendeu, e o que se perderia ao gravar.
 */
export interface CadDxfOpenReport {
	layerCount: number;
	entityCount: number;
	/** Entidades recusadas por referenciarem camada impossível. */
	skippedCount: number;
	blockCount: number;
	blockEntityCount: number;
	/** Camadas que o arquivo citava sem definir, criadas na leitura. */
	createdLayers: string[];
	/** Falhas locais de percurso, que não impediram a abertura. */
	errors: string[];
	loss: CadSaveLoss;
	/** Resumo de uma linha, pronto para mensagem. */
	summary: string;
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
