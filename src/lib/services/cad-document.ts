// Caminho relativo: src/lib/services/cad-document.ts

/**
 * \file src/lib/services/cad-document.ts
 * \brief Fronteira entre a UI e o kernel CAD compilado para WebAssembly.
 * \author Iago Leal
 * \date 2026-08-07
 *
 * É a **única** porta de acesso ao kernel. Componentes Svelte e rotas consomem
 * os contratos de `$lib/types/cad` e nunca importam `$lib/kernel`, conforme o
 * ADR 0001 — a mesma regra que já vale para o upstream `@mlightcad`.
 *
 * A tradução acontece aqui, uma vez. Se a forma do kernel mudar, muda este
 * arquivo, e não a interface inteira.
 */

import type {
	CadBounds,
	CadColor,
	CadDocumentSnapshot,
	CadEntity,
	CadEntityId,
	CadGeometry,
	CadHistoryState,
	CadLayer,
	CadLayerId,
	CadLayerSnapshot,
	CadDxfOpenReport,
	CadLoadReport,
	CadPaperSpaceLayout,
	CadPoint,
	CadSaveLoss,
	CadUnsupportedCount
} from '$lib/types/cad';

/** Superfície do módulo WebAssembly que este serviço consome. */
interface CadKernelModule {
	default: (options?: unknown) => Promise<unknown>;
	CadSession: new () => CadKernelSession;
}

/** Superfície da sessão exposta pelo kernel. */
interface CadKernelSession {
	layers(): unknown;
	entities(): unknown;
	entityCount(): number;
	boundingBox(): unknown;
	history(): unknown;
	createLayer(name: string): string;
	addLine(layer: string, startX: number, startY: number, endX: number, endY: number): string;
	removeEntity(entity: string): void;
	setLayerOff(layer: string, off: boolean): void;
	load(document: unknown): unknown;
	openDxf(bytes: Uint8Array): unknown;
	toDxf(): Uint8Array;
	saveLoss(): unknown;
	undo(): boolean;
	redo(): boolean;
}

let kernelModule: Promise<CadKernelModule> | null = null;

/**
 * Carrega e inicializa o módulo WebAssembly, uma vez por sessão do navegador.
 *
 * O import é dinâmico para que o `.wasm` não entre no bundle inicial: quem
 * apenas abre o app sem editar não paga por ele.
 */
async function loadKernel(): Promise<CadKernelModule> {
	kernelModule ??= (async () => {
		const module = (await import('$lib/kernel/pkg/neocad_wasm.js')) as unknown as CadKernelModule;
		await module.default();

		return module;
	})();

	return kernelModule;
}

/**
 * Falha ao interpretar um valor vindo do kernel.
 *
 * Diferente dos dados de `localStorage`, que são entrada não confiável e
 * merecem descarte silencioso, uma forma inesperada aqui é **defeito do
 * kernel**. Falhar alto evita que `undefined` se espalhe pela interface e o
 * sintoma apareça longe da causa.
 */
export class CadKernelContractError extends Error {
	constructor(message: string) {
		super(`Contrato do kernel violado: ${message}`);
		this.name = 'CadKernelContractError';
	}
}

function asRecord(value: unknown, context: string): Record<string, unknown> {
	if (typeof value !== 'object' || value === null) {
		throw new CadKernelContractError(`${context} deveria ser um objeto.`);
	}

	return value as Record<string, unknown>;
}

function asNumber(value: unknown, context: string): number {
	if (typeof value !== 'number' || !Number.isFinite(value)) {
		throw new CadKernelContractError(`${context} deveria ser um número finito.`);
	}

	return value;
}

function asString(value: unknown, context: string): string {
	if (typeof value !== 'string') {
		throw new CadKernelContractError(`${context} deveria ser texto.`);
	}

	return value;
}

function asBoolean(value: unknown, context: string): boolean {
	if (typeof value !== 'boolean') {
		throw new CadKernelContractError(`${context} deveria ser booleano.`);
	}

	return value;
}

function asArray(value: unknown, context: string): unknown[] {
	if (!Array.isArray(value)) {
		throw new CadKernelContractError(`${context} deveria ser uma lista.`);
	}

	return value;
}

/** Converte um ponto do kernel. */
export function toCadPoint(raw: unknown, context = 'ponto'): CadPoint {
	const record = asRecord(raw, context);

	return {
		x: asNumber(record.x, `${context}.x`),
		y: asNumber(record.y, `${context}.y`)
	};
}

/** Converte uma caixa envolvente do kernel. */
export function toCadBounds(raw: unknown, context = 'caixa envolvente'): CadBounds {
	const record = asRecord(raw, context);

	return {
		minX: asNumber(record.minX, `${context}.minX`),
		minY: asNumber(record.minY, `${context}.minY`),
		maxX: asNumber(record.maxX, `${context}.maxX`),
		maxY: asNumber(record.maxY, `${context}.maxY`)
	};
}

/** Converte uma cor do kernel. */
export function toCadColor(raw: unknown, context = 'cor'): CadColor {
	const record = asRecord(raw, context);
	const kind = asString(record.kind, `${context}.kind`);

	if (kind === 'byBlock' || kind === 'byLayer') {
		return { kind };
	}

	if (kind === 'index') {
		return { kind: 'index', index: asNumber(record.index, `${context}.index`) };
	}

	if (kind === 'rgb') {
		return {
			kind: 'rgb',
			red: asNumber(record.red, `${context}.red`),
			green: asNumber(record.green, `${context}.green`),
			blue: asNumber(record.blue, `${context}.blue`)
		};
	}

	throw new CadKernelContractError(`${context} tem tipo desconhecido: ${kind}.`);
}

/** Converte uma geometria do kernel. */
export function toCadGeometry(raw: unknown, context = 'geometria'): CadGeometry {
	const record = asRecord(raw, context);
	const kind = asString(record.kind, `${context}.kind`);

	switch (kind) {
		case 'line':
			return {
				kind: 'line',
				start: toCadPoint(record.start, `${context}.start`),
				end: toCadPoint(record.end, `${context}.end`)
			};
		case 'circle':
			return {
				kind: 'circle',
				center: toCadPoint(record.center, `${context}.center`),
				radius: asNumber(record.radius, `${context}.radius`)
			};
		case 'arc':
			return {
				kind: 'arc',
				center: toCadPoint(record.center, `${context}.center`),
				radius: asNumber(record.radius, `${context}.radius`),
				startAngle: asNumber(record.startAngle, `${context}.startAngle`),
				endAngle: asNumber(record.endAngle, `${context}.endAngle`)
			};
		case 'polyline':
			return {
				kind: 'polyline',
				vertices: asArray(record.vertices, `${context}.vertices`).map((vertex, index) =>
					toCadPoint(vertex, `${context}.vertices[${index}]`)
				),
				closed: asBoolean(record.closed, `${context}.closed`)
			};
		case 'text':
			return {
				kind: 'text',
				position: toCadPoint(record.position, `${context}.position`),
				content: asString(record.content, `${context}.content`),
				height: asNumber(record.height, `${context}.height`),
				rotation: asNumber(record.rotation, `${context}.rotation`)
			};
		case 'viewport':
			return {
				kind: 'viewport',
				center: toCadPoint(record.center, `${context}.center`),
				width: asNumber(record.width, `${context}.width`),
				height: asNumber(record.height, `${context}.height`),
				viewCenter: toCadPoint(record.viewCenter, `${context}.viewCenter`),
				viewHeight: asNumber(record.viewHeight, `${context}.viewHeight`),
				twist: asNumber(record.twist, `${context}.twist`),
				// A escala é derivada no kernel e chega pronta. Vem nula quando a
				// altura da vista é inválida — recalculá-la aqui faria as duas
				// contas divergirem, que é o defeito que o kernel evita.
				scale: record.scale == null ? null : asNumber(record.scale, `${context}.scale`),
				isOn: asBoolean(record.isOn, `${context}.isOn`),
				frozenLayers: asArray(record.frozenLayers, `${context}.frozenLayers`).map(
					(id, index) => asString(id, `${context}.frozenLayers[${index}]`) as CadLayerId
				)
			};
		default:
			throw new CadKernelContractError(`${context} tem tipo desconhecido: ${kind}.`);
	}
}

/** Converte o relatório de abertura de DXF vindo do kernel. */
export function toCadDxfOpenReport(raw: unknown, context = 'abertura DXF'): CadDxfOpenReport {
	const record = asRecord(raw, context);

	return {
		layerCount: asNumber(record.layerCount, `${context}.layerCount`),
		entityCount: asNumber(record.entityCount, `${context}.entityCount`),
		skippedCount: asNumber(record.skippedCount, `${context}.skippedCount`),
		blockCount: asNumber(record.blockCount, `${context}.blockCount`),
		blockEntityCount: asNumber(record.blockEntityCount, `${context}.blockEntityCount`),
		createdLayers: asArray(record.createdLayers, `${context}.createdLayers`).map((name, index) =>
			asString(name, `${context}.createdLayers[${index}]`)
		),
		errors: asArray(record.errors, `${context}.errors`).map((message, index) =>
			asString(message, `${context}.errors[${index}]`)
		),
		loss: toCadSaveLoss(record.loss, `${context}.loss`),
		summary: asString(record.summary, `${context}.summary`)
	};
}

/** Converte o relatório de perda de gravação vindo do kernel. */
export function toCadSaveLoss(raw: unknown, context = 'perda de gravação'): CadSaveLoss {
	const record = asRecord(raw, context);

	const unsupported: CadUnsupportedCount[] = asArray(
		record.unsupported,
		`${context}.unsupported`
	).map((item, index) => {
		const entry = asRecord(item, `${context}.unsupported[${index}]`);

		return {
			entityType: asString(entry.entityType, `${context}.unsupported[${index}].entityType`),
			count: asNumber(entry.count, `${context}.unsupported[${index}].count`)
		};
	});

	const paperSpace: CadPaperSpaceLayout[] = asArray(record.paperSpace, `${context}.paperSpace`).map(
		(item, index) => {
			const entry = asRecord(item, `${context}.paperSpace[${index}]`);

			return {
				name: asString(entry.name, `${context}.paperSpace[${index}].name`),
				entityCount: asNumber(entry.entityCount, `${context}.paperSpace[${index}].entityCount`)
			};
		}
	);

	return {
		unsupported,
		unsupportedCount: asNumber(record.unsupportedCount, `${context}.unsupportedCount`),
		paperSpace,
		paperSpaceCount: asNumber(record.paperSpaceCount, `${context}.paperSpaceCount`),
		xrefCount: asNumber(record.xrefCount, `${context}.xrefCount`),
		isLossless: asBoolean(record.isLossless, `${context}.isLossless`)
	};
}

/** Converte uma camada do kernel. */
export function toCadLayer(raw: unknown, context = 'camada'): CadLayer {
	const record = asRecord(raw, context);

	return {
		id: asString(record.id, `${context}.id`) as CadLayerId,
		name: asString(record.name, `${context}.name`),
		color: toCadColor(record.color, `${context}.color`),
		isVisible: asBoolean(record.visible, `${context}.visible`),
		isOff: asBoolean(record.off, `${context}.off`),
		isFrozen: asBoolean(record.frozen, `${context}.frozen`),
		isLocked: asBoolean(record.locked, `${context}.locked`)
	};
}

/** Converte uma entidade do kernel. */
export function toCadEntity(raw: unknown, context = 'entidade'): CadEntity {
	const record = asRecord(raw, context);

	return {
		id: asString(record.id, `${context}.id`) as CadEntityId,
		layerId: asString(record.layer, `${context}.layer`) as CadLayerId,
		geometry: toCadGeometry(record.geometry, `${context}.geometry`),
		bounds: toCadBounds(record.bounds, `${context}.bounds`)
	};
}

/** Converte o estado da pilha de comandos do kernel. */
export function toCadHistoryState(raw: unknown, context = 'histórico'): CadHistoryState {
	const record = asRecord(raw, context);
	const label = (value: unknown, field: string): string | null =>
		value === null || value === undefined ? null : asString(value, `${context}.${field}`);

	return {
		canUndo: asBoolean(record.canUndo, `${context}.canUndo`),
		canRedo: asBoolean(record.canRedo, `${context}.canRedo`),
		undoLabel: label(record.undoName, 'undoName'),
		redoLabel: label(record.redoName, 'redoName'),
		undoDepth: asNumber(record.undoDepth, `${context}.undoDepth`),
		redoDepth: asNumber(record.redoDepth, `${context}.redoDepth`)
	};
}

/** Converte a lista de camadas do kernel. */
export function toCadLayers(raw: unknown): CadLayer[] {
	return asArray(raw, 'lista de camadas').map((layer, index) =>
		toCadLayer(layer, `camada[${index}]`)
	);
}

/** Converte a lista de entidades do kernel. */
export function toCadEntities(raw: unknown): CadEntity[] {
	return asArray(raw, 'lista de entidades').map((entity, index) =>
		toCadEntity(entity, `entidade[${index}]`)
	);
}

// -- Extração do documento do upstream ---------------------------------------
//
// A leitura acontece sobre a **forma** dos objetos do `@mlightcad/data-model`,
// não sobre os seus tipos. Duas razões:
//
// 1. As declarações do upstream expõem `entity.type` como `string`, mas não
//    declaram quais valores ele assume. Depender de constantes não declaradas
//    seria adivinhação, e uma renomeação silenciosa lá viraria perda de
//    entidades aqui.
// 2. Detectar pela presença dos acessores característicos torna a conversão
//    exercitável com objetos sintéticos, sem carregar o upstream nem um
//    navegador.

function readNumber(value: unknown): number | null {
	return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

/** Lê um ponto do upstream, que usa `AcGePoint2d`/`AcGePoint3d`. */
function readUpstreamPoint(value: unknown): CadPoint | null {
	if (typeof value !== 'object' || value === null) {
		return null;
	}

	const record = value as Record<string, unknown>;
	const x = readNumber(record.x);
	const y = readNumber(record.y);

	return x === null || y === null ? null : { x, y };
}

/**
 * Converte a geometria de uma entidade do upstream.
 *
 * Devolve `null` quando a forma não corresponde a nenhuma geometria que o kernel
 * modela — polilinhas com abaulamento, hachuras, cotas, referências de bloco.
 *
 * A ordem de teste importa: um arco também tem centro e raio, então precisa ser
 * reconhecido **antes** do círculo.
 */
export function toCadGeometryFromUpstream(entity: unknown): CadGeometry | null {
	if (typeof entity !== 'object' || entity === null) {
		return null;
	}

	const record = entity as Record<string, unknown>;

	const start = readUpstreamPoint(record.startPoint);
	const end = readUpstreamPoint(record.endPoint);
	if (start && end) {
		return { kind: 'line', start, end };
	}

	const center = readUpstreamPoint(record.center);
	const radius = readNumber(record.radius);
	if (center && radius !== null) {
		const startAngle = readNumber(record.startAngle);
		const endAngle = readNumber(record.endAngle);

		if (startAngle !== null && endAngle !== null) {
			return { kind: 'arc', center, radius, startAngle, endAngle };
		}

		return { kind: 'circle', center, radius };
	}

	const vertexCount = readNumber(record.numberOfVertices);
	if (vertexCount !== null && typeof record.getPoint2dAt === 'function') {
		const readVertex = record.getPoint2dAt as (index: number) => unknown;
		const vertices: CadPoint[] = [];

		for (let index = 0; index < vertexCount; index += 1) {
			const vertex = readUpstreamPoint(readVertex.call(entity, index));

			// Um vértice ilegível invalida a polilinha inteira: desenhá-la sem
			// ele mudaria a forma em silêncio.
			if (!vertex) {
				return null;
			}

			vertices.push(vertex);
		}

		return { kind: 'polyline', vertices, closed: record.closed === true };
	}

	const position = readUpstreamPoint(record.position);
	const height = readNumber(record.height);
	if (position && height !== null && typeof record.textString === 'string') {
		return {
			kind: 'text',
			position,
			content: record.textString,
			height,
			rotation: readNumber(record.rotation) ?? 0
		};
	}

	return null;
}

/** Converte a cor de uma camada do upstream (`AcCmColor`). */
export function toCadColorFromUpstream(value: unknown): CadColor {
	if (typeof value === 'object' && value !== null) {
		const record = value as Record<string, unknown>;
		const red = readNumber(record.red);
		const green = readNumber(record.green);
		const blue = readNumber(record.blue);

		if (red !== null && green !== null && blue !== null) {
			return { kind: 'rgb', red, green, blue };
		}

		const index = readNumber(record.colorIndex);
		if (index !== null) {
			// A paleta ACI vai de 0 a 256, e os extremos são herança, não cor.
			if (index === 0) return { kind: 'byBlock' };
			if (index === 256) return { kind: 'byLayer' };
			if (index >= 1 && index <= 255) return { kind: 'index', index };
		}
	}

	// Índice 7 é o padrão de desenho novo, e é o que um arquivo sem cor
	// declarada significa.
	return { kind: 'index', index: 7 };
}

/** Converte uma camada do upstream (`AcDbLayerTableRecord`). */
export function toCadLayerSnapshot(record: unknown): CadLayerSnapshot | null {
	if (typeof record !== 'object' || record === null) {
		return null;
	}

	const layer = record as Record<string, unknown>;
	if (typeof layer.name !== 'string' || layer.name.trim().length === 0) {
		return null;
	}

	return {
		name: layer.name,
		color: toCadColorFromUpstream(layer.color),
		isOff: layer.isOff === true,
		isFrozen: layer.isFrozen === true,
		isLocked: layer.isLocked === true
	};
}

/**
 * Monta o retrato de um documento do upstream, pronto para carregar no kernel.
 *
 * Entidades que o kernel ainda não modela entram em `unsupported` e **não
 * interrompem a extração**: um arquivo real traz hachuras, cotas e referências
 * de bloco, e deixar de abrir por causa delas seria pior do que abrir
 * parcialmente.
 */
export function buildDocumentSnapshot(
	layers: Iterable<unknown>,
	entities: Iterable<unknown>
): CadDocumentSnapshot {
	const snapshot: CadDocumentSnapshot = { layers: [], entities: [], unsupported: [] };

	for (const record of layers) {
		const layer = toCadLayerSnapshot(record);

		if (layer) {
			snapshot.layers.push(layer);
		}
	}

	for (const entity of entities) {
		const record = (entity ?? {}) as Record<string, unknown>;
		const layerName = typeof record.layer === 'string' ? record.layer : '0';
		const geometry = toCadGeometryFromUpstream(entity);

		if (geometry) {
			snapshot.entities.push({ layerName, geometry });
			continue;
		}

		snapshot.unsupported.push({
			type: typeof record.type === 'string' ? record.type : 'desconhecido',
			layerName
		});
	}

	return snapshot;
}

/**
 * Documento CAD do kernel, na forma que a UI consome.
 *
 * Envolve a sessão WebAssembly e devolve sempre contratos NeoCAD. Nenhum tipo
 * do kernel atravessa esta classe.
 */
export class CadDocument {
	private constructor(private readonly session: CadKernelSession) {}

	/** Cria um documento vazio, carregando o kernel na primeira chamada. */
	static async create(): Promise<CadDocument> {
		const { CadSession } = await loadKernel();

		return new CadDocument(new CadSession());
	}

	/** Camadas do documento, em ordem alfabética. */
	listLayers(): CadLayer[] {
		return toCadLayers(this.session.layers());
	}

	/** Entidades do espaço-modelo, na ordem de desenho. */
	listEntities(): CadEntity[] {
		return toCadEntities(this.session.entities());
	}

	/** Quantidade de entidades do documento. */
	countEntities(): number {
		return this.session.entityCount();
	}

	/** Extensão do espaço-modelo, ou `null` quando não há entidades. */
	getBounds(): CadBounds | null {
		const raw = this.session.boundingBox();

		return raw === null || raw === undefined ? null : toCadBounds(raw);
	}

	/** Estado da pilha de comandos. */
	getHistory(): CadHistoryState {
		return toCadHistoryState(this.session.history());
	}

	/** Cria uma camada e devolve seu identificador. */
	createLayer(name: string): CadLayerId {
		return this.session.createLayer(name) as CadLayerId;
	}

	/** Desenha um segmento de reta como uma ação desfazível. */
	drawLine(layerId: CadLayerId, start: CadPoint, end: CadPoint): CadEntityId {
		return this.session.addLine(layerId, start.x, start.y, end.x, end.y) as CadEntityId;
	}

	/** Apaga uma entidade como uma ação desfazível. */
	eraseEntity(entityId: CadEntityId): void {
		this.session.removeEntity(entityId);
	}

	/** Liga ou desliga uma camada como uma ação desfazível. */
	setLayerOff(layerId: CadLayerId, isOff: boolean): void {
		this.session.setLayerOff(layerId, isOff);
	}

	/**
	 * Substitui o documento pelo retrato extraído do upstream.
	 *
	 * O histórico é zerado pelo kernel: desfazer para antes da abertura não faz
	 * sentido.
	 */
	load(snapshot: CadDocumentSnapshot): CadLoadReport {
		const raw = this.session.load({
			layers: snapshot.layers.map((layer) => ({
				name: layer.name,
				color: layer.color,
				off: layer.isOff,
				frozen: layer.isFrozen,
				locked: layer.isLocked
			})),
			entities: snapshot.entities
		});
		const record = asRecord(raw, 'relatório de carregamento');

		return {
			layerCount: asNumber(record.layerCount, 'relatório.layerCount'),
			entityCount: asNumber(record.entityCount, 'relatório.entityCount'),
			skippedCount: asNumber(record.skippedCount, 'relatório.skippedCount'),
			unsupportedCount: snapshot.unsupported.length
		};
	}

	/**
	 * Lê um DXF com o kernel, substituindo o documento.
	 *
	 * É a leitura **própria**, que não passa pelo upstream. Só o espaço-modelo
	 * entra no documento; o que ficou de fora vem em
	 * [`CadDxfOpenReport.loss`], para a interface poder dizer o que existe e
	 * ainda não é exibido.
	 */
	openDxf(content: ArrayBuffer): CadDxfOpenReport {
		return toCadDxfOpenReport(this.session.openDxf(new Uint8Array(content)));
	}

	/**
	 * Serializa o documento para DXF.
	 *
	 * A saída é determinística: o mesmo documento produz os mesmos bytes, o que
	 * é o que torna um desenho versionável sem ruído (ADR 0004).
	 */
	toDxf(): Uint8Array {
		return this.session.toDxf();
	}

	/**
	 * O que uma gravação descartaria do desenho aberto.
	 *
	 * Enquanto a abertura passar pelo upstream (até o MT-K2-12), o kernel só
	 * conhece o que recebeu pelo retrato: esta consulta cobre a parte que a
	 * leitura nativa já sabe relatar, e a rota completa com o que a extração
	 * contou.
	 */
	getSaveLoss(): CadSaveLoss {
		return toCadSaveLoss(this.session.saveLoss());
	}

	/** Desfaz a última ação. Devolve `false` se não houver o que desfazer. */
	undo(): boolean {
		return this.session.undo();
	}

	/** Refaz a última ação desfeita. Devolve `false` se não houver o que refazer. */
	redo(): boolean {
		return this.session.redo();
	}
}
