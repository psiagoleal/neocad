// Caminho relativo: src/lib/services/cad-document.spec.ts

/**
 * \file src/lib/services/cad-document.spec.ts
 * \brief Testes da conversão entre as formas do kernel e os contratos NeoCAD.
 * \author Iago Leal
 * \date 2026-08-07
 *
 * Os conversores são funções puras justamente para poderem ser exercitados sem
 * carregar o WebAssembly: o pacote `--target web` busca o `.wasm` por `fetch`,
 * que não resolve `file://` no Node. Testar a tradução não exige o kernel — só
 * exige conhecer a forma que ele produz.
 */

import { describe, expect, it } from 'vitest';
import {
	buildDocumentSnapshot,
	CadKernelContractError,
	toCadBounds,
	toCadColorFromUpstream,
	toCadGeometryFromUpstream,
	toCadLayerSnapshot,
	toCadColor,
	toCadEntities,
	toCadEntity,
	toCadGeometry,
	toCadHistoryState,
	toCadLayer,
	toCadLayers,
	toCadPoint,
	toCadSaveLoss
} from './cad-document';

/** Camada como o kernel a serializa (ver `LayerView` em neocad-wasm). */
const camadaDoKernel = {
	id: '4294967297',
	name: 'Parede',
	color: { kind: 'index', index: 7 },
	visible: true,
	off: false,
	frozen: false,
	locked: false
};

/** Entidade como o kernel a serializa (ver `EntityView` em neocad-wasm). */
const entidadeDoKernel = {
	id: '8589934593',
	layer: '4294967297',
	geometry: {
		kind: 'line',
		start: { x: 0, y: 0 },
		end: { x: 10, y: 5 }
	},
	bounds: { minX: 0, minY: 0, maxX: 10, maxY: 5 }
};

describe('toCadPoint', () => {
	it('converte coordenadas', () => {
		expect(toCadPoint({ x: 1.5, y: -2 })).toEqual({ x: 1.5, y: -2 });
	});

	it('recusa coordenada não finita', () => {
		expect(() => toCadPoint({ x: Number.NaN, y: 0 })).toThrow(CadKernelContractError);
		expect(() => toCadPoint({ x: 0, y: Number.POSITIVE_INFINITY })).toThrow(CadKernelContractError);
	});

	it('recusa valor que não é objeto', () => {
		expect(() => toCadPoint(null)).toThrow(CadKernelContractError);
		expect(() => toCadPoint('0,0')).toThrow(CadKernelContractError);
	});
});

describe('toCadBounds', () => {
	it('converte a caixa envolvente', () => {
		expect(toCadBounds({ minX: -1, minY: -2, maxX: 3, maxY: 4 })).toEqual({
			minX: -1,
			minY: -2,
			maxX: 3,
			maxY: 4
		});
	});

	it('recusa caixa com campo ausente', () => {
		expect(() => toCadBounds({ minX: 0, minY: 0, maxX: 1 })).toThrow(CadKernelContractError);
	});
});

describe('toCadColor', () => {
	it('converte cor por índice', () => {
		expect(toCadColor({ kind: 'index', index: 3 })).toEqual({ kind: 'index', index: 3 });
	});

	it('converte cor verdadeira', () => {
		expect(toCadColor({ kind: 'rgb', red: 200, green: 30, blue: 10 })).toEqual({
			kind: 'rgb',
			red: 200,
			green: 30,
			blue: 10
		});
	});

	it('recusa tipo de cor desconhecido', () => {
		expect(() => toCadColor({ kind: 'cmyk' })).toThrow(/tipo desconhecido/);
	});
});

describe('toCadGeometry', () => {
	it('converte linha', () => {
		const geometria = toCadGeometry({
			kind: 'line',
			start: { x: 0, y: 0 },
			end: { x: 1, y: 1 }
		});

		expect(geometria).toEqual({
			kind: 'line',
			start: { x: 0, y: 0 },
			end: { x: 1, y: 1 }
		});
	});

	it('converte círculo', () => {
		expect(toCadGeometry({ kind: 'circle', center: { x: 2, y: 2 }, radius: 5 })).toEqual({
			kind: 'circle',
			center: { x: 2, y: 2 },
			radius: 5
		});
	});

	it('converte arco preservando os ângulos', () => {
		expect(
			toCadGeometry({
				kind: 'arc',
				center: { x: 0, y: 0 },
				radius: 1,
				startAngle: 0.5,
				endAngle: 2.5
			})
		).toEqual({
			kind: 'arc',
			center: { x: 0, y: 0 },
			radius: 1,
			startAngle: 0.5,
			endAngle: 2.5
		});
	});

	it('converte polilinha com todos os vértices', () => {
		const geometria = toCadGeometry({
			kind: 'polyline',
			vertices: [
				{ x: 0, y: 0 },
				{ x: 1, y: 0 },
				{ x: 1, y: 1 }
			],
			closed: true
		});

		expect(geometria).toEqual({
			kind: 'polyline',
			vertices: [
				{ x: 0, y: 0 },
				{ x: 1, y: 0 },
				{ x: 1, y: 1 }
			],
			closed: true
		});
	});

	it('converte texto', () => {
		expect(
			toCadGeometry({
				kind: 'text',
				position: { x: 1, y: 2 },
				content: 'Corte AA',
				height: 2.5,
				rotation: 0
			})
		).toEqual({
			kind: 'text',
			position: { x: 1, y: 2 },
			content: 'Corte AA',
			height: 2.5,
			rotation: 0
		});
	});

	it('aponta o vértice defeituoso da polilinha', () => {
		expect(() =>
			toCadGeometry({
				kind: 'polyline',
				vertices: [{ x: 0, y: 0 }, { x: 1 }],
				closed: false
			})
		).toThrow(/vertices\[1\]\.y/);
	});

	it('recusa geometria de tipo desconhecido', () => {
		expect(() => toCadGeometry({ kind: 'spline' })).toThrow(/tipo desconhecido/);
	});
});

describe('toCadLayer', () => {
	it('converte a camada e renomeia os estados para a convenção da UI', () => {
		expect(toCadLayer(camadaDoKernel)).toEqual({
			id: '4294967297',
			name: 'Parede',
			color: { kind: 'index', index: 7 },
			isVisible: true,
			isOff: false,
			isFrozen: false,
			isLocked: false
		});
	});

	it('preserva camada desligada como não visível', () => {
		const convertida = toCadLayer({ ...camadaDoKernel, visible: false, off: true });

		expect(convertida.isVisible).toBe(false);
		expect(convertida.isOff).toBe(true);
	});

	it('recusa camada sem identificador', () => {
		const semId = { ...camadaDoKernel, id: undefined };

		expect(() => toCadLayer(semId)).toThrow(CadKernelContractError);
	});
});

describe('toCadEntity', () => {
	it('converte a entidade e renomeia a referência de camada', () => {
		expect(toCadEntity(entidadeDoKernel)).toEqual({
			id: '8589934593',
			layerId: '4294967297',
			geometry: {
				kind: 'line',
				start: { x: 0, y: 0 },
				end: { x: 10, y: 5 }
			},
			bounds: { minX: 0, minY: 0, maxX: 10, maxY: 5 }
		});
	});

	it('recusa entidade com geometria malformada, apontando o campo', () => {
		expect(() =>
			toCadEntity({ ...entidadeDoKernel, geometry: { kind: 'line', start: { x: 0, y: 0 } } })
		).toThrow(/entidade\.geometry\.end/);
	});
});

describe('toCadHistoryState', () => {
	it('converte o estado com ações disponíveis', () => {
		expect(
			toCadHistoryState({
				canUndo: true,
				canRedo: false,
				undoName: 'Desenhar linha',
				redoName: null,
				undoDepth: 3,
				redoDepth: 0
			})
		).toEqual({
			canUndo: true,
			canRedo: false,
			undoLabel: 'Desenhar linha',
			redoLabel: null,
			undoDepth: 3,
			redoDepth: 0
		});
	});

	it('trata ausência de rótulo como nulo', () => {
		const estado = toCadHistoryState({
			canUndo: false,
			canRedo: false,
			undoDepth: 0,
			redoDepth: 0
		});

		expect(estado.undoLabel).toBeNull();
		expect(estado.redoLabel).toBeNull();
	});
});

describe('conversão de listas', () => {
	it('converte todas as camadas preservando a ordem', () => {
		const camadas = toCadLayers([
			camadaDoKernel,
			{ ...camadaDoKernel, id: '4294967298', name: 'Cotas' }
		]);

		expect(camadas.map((camada) => camada.name)).toEqual(['Parede', 'Cotas']);
	});

	it('converte todas as entidades preservando a ordem de desenho', () => {
		const entidades = toCadEntities([entidadeDoKernel, { ...entidadeDoKernel, id: '8589934594' }]);

		expect(entidades.map((entidade) => entidade.id)).toEqual(['8589934593', '8589934594']);
	});

	it('lista vazia produz lista vazia', () => {
		expect(toCadLayers([])).toEqual([]);
		expect(toCadEntities([])).toEqual([]);
	});

	it('aponta o índice do item defeituoso', () => {
		expect(() => toCadLayers([camadaDoKernel, { ...camadaDoKernel, name: 42 }])).toThrow(
			/camada\[1\]\.name/
		);
	});

	it('recusa valor que não é lista', () => {
		expect(() => toCadEntities({ length: 0 })).toThrow(CadKernelContractError);
	});
});

// -- Extração a partir do upstream -------------------------------------------
//
// Os objetos abaixo imitam a forma do `@mlightcad/data-model`: `AcDbLine` expõe
// `startPoint`/`endPoint`, `AcDbArc` acrescenta ângulos a centro e raio,
// `AcDbPolyline` responde a `numberOfVertices`/`getPoint2dAt`, e todos carregam
// `layer` como **nome**. Imitar a forma, e não importar os tipos, é o que
// permite exercitar a conversão sem navegador.

const linhaDoUpstream = {
	type: 'AcDbLine',
	layer: 'Parede',
	startPoint: { x: 0, y: 0, z: 0 },
	endPoint: { x: 10, y: 5, z: 0 }
};

const arcoDoUpstream = {
	type: 'AcDbArc',
	layer: '0',
	center: { x: 1, y: 1, z: 0 },
	radius: 3,
	startAngle: 0.25,
	endAngle: 1.75
};

const circuloDoUpstream = {
	type: 'AcDbCircle',
	layer: '0',
	center: { x: 2, y: 2, z: 0 },
	radius: 4
};

const polilinhaDoUpstream = {
	type: 'AcDbPolyline',
	layer: 'Cotas',
	numberOfVertices: 3,
	closed: true,
	getPoint2dAt(index: number) {
		return [
			{ x: 0, y: 0 },
			{ x: 4, y: 0 },
			{ x: 4, y: 3 }
		][index];
	}
};

const textoDoUpstream = {
	type: 'AcDbText',
	layer: '0',
	position: { x: 1, y: 2, z: 0 },
	textString: 'Corte AA',
	height: 2.5,
	rotation: 0.5
};

const camadaDoUpstream = {
	name: 'Parede',
	color: { colorIndex: 3, red: undefined, green: undefined, blue: undefined },
	isOff: false,
	isFrozen: false,
	isLocked: false
};

describe('toCadGeometryFromUpstream', () => {
	it('reconhece linha por startPoint e endPoint', () => {
		expect(toCadGeometryFromUpstream(linhaDoUpstream)).toEqual({
			kind: 'line',
			start: { x: 0, y: 0 },
			end: { x: 10, y: 5 }
		});
	});

	it('reconhece arco antes de círculo, pois ambos têm centro e raio', () => {
		expect(toCadGeometryFromUpstream(arcoDoUpstream)).toEqual({
			kind: 'arc',
			center: { x: 1, y: 1 },
			radius: 3,
			startAngle: 0.25,
			endAngle: 1.75
		});
	});

	it('reconhece círculo quando não há ângulos', () => {
		expect(toCadGeometryFromUpstream(circuloDoUpstream)).toEqual({
			kind: 'circle',
			center: { x: 2, y: 2 },
			radius: 4
		});
	});

	it('lê todos os vértices da polilinha', () => {
		expect(toCadGeometryFromUpstream(polilinhaDoUpstream)).toEqual({
			kind: 'polyline',
			vertices: [
				{ x: 0, y: 0 },
				{ x: 4, y: 0 },
				{ x: 4, y: 3 }
			],
			closed: true
		});
	});

	it('descarta a polilinha inteira quando um vértice é ilegível', () => {
		const defeituosa = {
			...polilinhaDoUpstream,
			getPoint2dAt: (index: number) => (index === 1 ? null : { x: 0, y: 0 })
		};

		expect(
			toCadGeometryFromUpstream(defeituosa),
			'desenhá-la sem o vértice mudaria a forma em silêncio'
		).toBeNull();
	});

	it('reconhece texto e assume rotação zero quando ausente', () => {
		expect(toCadGeometryFromUpstream(textoDoUpstream)).toEqual({
			kind: 'text',
			position: { x: 1, y: 2 },
			content: 'Corte AA',
			height: 2.5,
			rotation: 0.5
		});

		const semRotacao = { ...textoDoUpstream, rotation: undefined };
		expect(toCadGeometryFromUpstream(semRotacao)).toMatchObject({ rotation: 0 });
	});

	it('devolve nulo para entidade que o kernel ainda não modela', () => {
		expect(toCadGeometryFromUpstream({ type: 'AcDbHatch', layer: '0' })).toBeNull();
		expect(toCadGeometryFromUpstream(null)).toBeNull();
	});

	it('ignora a coordenada z das entidades 3D do upstream', () => {
		const geometria = toCadGeometryFromUpstream({
			...linhaDoUpstream,
			startPoint: { x: 0, y: 0, z: 50 }
		});

		expect(geometria).toEqual({
			kind: 'line',
			start: { x: 0, y: 0 },
			end: { x: 10, y: 5 }
		});
	});
});

describe('toCadColorFromUpstream', () => {
	it('lê cor verdadeira quando os componentes existem', () => {
		expect(toCadColorFromUpstream({ red: 10, green: 20, blue: 30 })).toEqual({
			kind: 'rgb',
			red: 10,
			green: 20,
			blue: 30
		});
	});

	it('lê índice ACI quando não há cor verdadeira', () => {
		expect(toCadColorFromUpstream({ colorIndex: 5 })).toEqual({ kind: 'index', index: 5 });
	});

	it('assume o índice 7 quando a cor não é legível', () => {
		expect(toCadColorFromUpstream(undefined)).toEqual({ kind: 'index', index: 7 });
		expect(toCadColorFromUpstream({})).toEqual({ kind: 'index', index: 7 });
	});
});

describe('toCadLayerSnapshot', () => {
	it('converte a camada com seus estados', () => {
		expect(toCadLayerSnapshot({ ...camadaDoUpstream, isFrozen: true })).toEqual({
			name: 'Parede',
			color: { kind: 'index', index: 3 },
			isOff: false,
			isFrozen: true,
			isLocked: false
		});
	});

	it('descarta camada sem nome utilizável', () => {
		expect(toCadLayerSnapshot({ ...camadaDoUpstream, name: '  ' })).toBeNull();
		expect(toCadLayerSnapshot(null)).toBeNull();
	});
});

describe('buildDocumentSnapshot', () => {
	it('converte um documento sintético inteiro', () => {
		const snapshot = buildDocumentSnapshot(
			[camadaDoUpstream, { ...camadaDoUpstream, name: '0' }],
			[linhaDoUpstream, arcoDoUpstream, polilinhaDoUpstream, textoDoUpstream]
		);

		expect(snapshot.layers.map((layer) => layer.name)).toEqual(['Parede', '0']);
		expect(snapshot.entities).toHaveLength(4);
		expect(snapshot.entities.map((entity) => entity.geometry.kind)).toEqual([
			'line',
			'arc',
			'polyline',
			'text'
		]);
		expect(snapshot.unsupported).toHaveLength(0);
	});

	it('preserva a camada de cada entidade pelo nome', () => {
		const snapshot = buildDocumentSnapshot([], [linhaDoUpstream, polilinhaDoUpstream]);

		expect(snapshot.entities.map((entity) => entity.layerName)).toEqual(['Parede', 'Cotas']);
	});

	it('registra o não suportado sem interromper a extração', () => {
		const snapshot = buildDocumentSnapshot(
			[camadaDoUpstream],
			[
				{ type: 'AcDbHatch', layer: 'Parede' },
				linhaDoUpstream,
				{ type: 'AcDbDimension', layer: 'Cotas' }
			]
		);

		expect(
			snapshot.entities,
			'a linha entre duas entidades não suportadas precisa sobreviver'
		).toHaveLength(1);
		expect(snapshot.unsupported).toEqual([
			{ type: 'AcDbHatch', layerName: 'Parede' },
			{ type: 'AcDbDimension', layerName: 'Cotas' }
		]);
	});

	it('assume a camada 0 quando a entidade não declara camada', () => {
		const snapshot = buildDocumentSnapshot([], [{ ...linhaDoUpstream, layer: undefined }]);

		expect(snapshot.entities[0].layerName).toBe('0');
	});

	it('documento vazio produz retrato vazio', () => {
		expect(buildDocumentSnapshot([], [])).toEqual({
			layers: [],
			entities: [],
			unsupported: []
		});
	});

	it('aceita iteradores, e não apenas listas', () => {
		function* camadas() {
			yield camadaDoUpstream;
		}
		function* entidades() {
			yield linhaDoUpstream;
		}

		const snapshot = buildDocumentSnapshot(camadas(), entidades());

		expect(snapshot.layers).toHaveLength(1);
		expect(snapshot.entities).toHaveLength(1);
	});
});

describe('CadKernelContractError', () => {
	it('identifica a origem do defeito na mensagem', () => {
		const erro = new CadKernelContractError('camada.id deveria ser texto.');

		expect(erro.name).toBe('CadKernelContractError');
		expect(erro.message).toContain('Contrato do kernel violado');
	});
});

describe('relatório de perda de gravação', () => {
	const perdaCompleta = {
		unsupported: [{ entityType: 'HATCH', count: 3 }],
		unsupportedCount: 3,
		paperSpace: [{ name: 'Prancha A1', entityCount: 19 }],
		paperSpaceCount: 19,
		xrefCount: 1,
		isLossless: false
	};

	it('converte o relatório do kernel', () => {
		const perda = toCadSaveLoss(perdaCompleta);

		expect(perda.unsupported).toEqual([{ entityType: 'HATCH', count: 3 }]);
		expect(perda.paperSpace).toEqual([{ name: 'Prancha A1', entityCount: 19 }]);
		expect(perda.xrefCount).toBe(1);
		expect(perda.isLossless).toBe(false);
	});

	it('aceita o caso sem perda alguma', () => {
		const perda = toCadSaveLoss({
			unsupported: [],
			unsupportedCount: 0,
			paperSpace: [],
			paperSpaceCount: 0,
			xrefCount: 0,
			isLossless: true
		});

		expect(perda.isLossless).toBe(true);
		expect(perda.unsupported).toHaveLength(0);
	});

	it('falha alto quando a forma do kernel não bate', () => {
		// Forma inesperada aqui é defeito do kernel, não entrada não confiável:
		// deixar passar faria o sintoma aparecer longe da causa.
		expect(() => toCadSaveLoss({ ...perdaCompleta, isLossless: 'não' })).toThrow(
			CadKernelContractError
		);
		expect(() => toCadSaveLoss({ ...perdaCompleta, paperSpace: null })).toThrow(
			CadKernelContractError
		);
	});
});
