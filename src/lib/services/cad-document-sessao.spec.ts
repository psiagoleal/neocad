// Caminho relativo: src/lib/services/cad-document-sessao.spec.ts

/**
 * \file src/lib/services/cad-document-sessao.spec.ts
 * \brief Testes das operações e consultas de K3 no serviço, sobre sessão simulada.
 * \author Iago Leal
 * \date 2026-10-30
 *
 * O `.wasm` não carrega no Node (ver `cad-document.spec.ts`); o comportamento real
 * das operações é provado nos testes de host de `neocad-wasm`. Aqui se prova o que é
 * do serviço: repasse, conversão validada, erro tipado e o documento envenenado
 * depois de uma falha interna (spec 06, W1–W4b).
 */

import { describe, expect, it, vi } from 'vitest';
import {
	CadDocument,
	CadKernelContractError,
	CadKernelFailureError,
	CadKernelOperationError,
	type CadKernelSession
} from './cad-document';
import type { CadEntityId, CadLayerId, CadSnapMode } from '$lib/types/cad';

type SessaoFalsa = Record<string, ReturnType<typeof vi.fn>>;

/** Sessão simulada: todo método devolve um valor neutro, salvo o que `troca` sobrescreve. */
function sessaoFalsa(troca: Record<string, unknown> = {}): SessaoFalsa {
	const neutros: Record<string, () => unknown> = {
		layers: () => [],
		entities: () => [],
		entityCount: () => 0,
		boundingBox: () => null,
		history: () => ({}),
		createLayer: () => '1',
		addLine: () => '1',
		removeEntity: () => undefined,
		setLayerOff: () => undefined,
		load: () => ({}),
		openDxf: () => ({}),
		toDxf: () => new Uint8Array(),
		saveLoss: () => ({}),
		undo: () => false,
		redo: () => false,
		moveEntities: () => undefined,
		copyEntities: () => [],
		rotateEntities: () => undefined,
		mirrorEntities: () => undefined,
		scaleEntities: () => undefined,
		offsetEntity: () => [],
		trimEntity: () => [],
		extendEntity: () => undefined,
		filletEntities: () => undefined,
		chamferEntities: () => undefined,
		intersections: () => [],
		entitiesInBox: () => [],
		hitTest: () => [],
		snap: () => null,
		prepareScreen: () => undefined,
		screenSpaces: () => [],
		screenOrigin: () => new Float64Array([0, 0]),
		screenLayers: () => [],
		screenBlock: () => new Float32Array(),
		updateScreen: () => [],
		screenQuery: () => []
	};
	const sessao: SessaoFalsa = {};

	for (const [nome, neutro] of Object.entries(neutros)) {
		sessao[nome] = vi.fn(neutro);
	}

	return { ...sessao, ...(troca as SessaoFalsa) };
}

function documentoSobre(sessao: SessaoFalsa): CadDocument {
	return CadDocument.fromSession(sessao as unknown as CadKernelSession);
}

const id = (texto: string) => texto as CadEntityId;

describe('CadDocument: operações e consultas (W1, W2)', () => {
	it('moveEntities repassa ids e o deslocamento achatado', () => {
		const sessao = sessaoFalsa({
			moveEntities: vi.fn()
		});
		const doc = documentoSobre(sessao);
		const ids = [id('1'), id('2')];
		const delta = { x: 3, y: 4 };

		doc.moveEntities(ids, delta);

		expect(sessao.moveEntities).toHaveBeenCalledWith(ids, delta.x, delta.y);
	});

	it('copyEntities devolve os ids que a sessão devolve', () => {
		const sessao = sessaoFalsa({
			copyEntities: vi.fn(() => ['7', '8'])
		});
		const doc = documentoSobre(sessao);
		const ids = [id('1'), id('2')];
		const delta = { x: 3, y: 4 };

		const result = doc.copyEntities(ids, delta);

		expect(result).toEqual(['7', '8']);
		expect(sessao.copyEntities).toHaveBeenCalledWith(ids, delta.x, delta.y);
	});

	it('rotateEntities, mirrorEntities e scaleEntities repassam pivô/ponto achatado e o número', () => {
		const sessao = sessaoFalsa({
			rotateEntities: vi.fn(),
			mirrorEntities: vi.fn(),
			scaleEntities: vi.fn()
		});
		const doc = documentoSobre(sessao);
		const ids = [id('1'), id('2')];
		const pivot = { x: 10, y: 20 };
		const angle = Math.PI / 2;

		doc.rotateEntities(ids, pivot, angle);
		doc.mirrorEntities(ids, pivot, angle);
		doc.scaleEntities(ids, pivot, 2.0);

		expect(sessao.rotateEntities).toHaveBeenCalledWith(ids, pivot.x, pivot.y, angle);
		expect(sessao.mirrorEntities).toHaveBeenCalledWith(ids, pivot.x, pivot.y, angle);
		expect(sessao.scaleEntities).toHaveBeenCalledWith(ids, pivot.x, pivot.y, 2.0);
	});

	it('offsetEntity e trimEntity repassam os argumentos achatados e devolvem os ids da sessão', () => {
		const sessao = sessaoFalsa({
			offsetEntity: vi.fn(() => ['9']),
			trimEntity: vi.fn(() => ['10'])
		});
		const doc = documentoSobre(sessao);
		const alvo = id('1');
		const distance = 5;
		const side = { x: 1, y: 1 };

		const offsetResult = doc.offsetEntity(alvo, distance, side);
		const trimResult = doc.trimEntity(alvo, [id('a'), id('b')], { x: 2, y: 2 }, 0.5);

		expect(offsetResult).toEqual(['9']);
		expect(sessao.offsetEntity).toHaveBeenCalledWith(alvo, distance, side.x, side.y);
		expect(trimResult).toEqual(['10']);
		expect(sessao.trimEntity).toHaveBeenCalledWith(alvo, [id('a'), id('b')], 2, 2, 0.5);
	});

	it('extendEntity repassa as fronteiras, o clique achatado e a captura', () => {
		const sessao = sessaoFalsa({
			extendEntity: vi.fn()
		});
		const doc = documentoSobre(sessao);
		const alvo = id('1');
		const boundaries = [id('a'), id('b')];
		const pick = { x: 3, y: 4 };
		const capture = 0.5;

		doc.extendEntity(alvo, boundaries, pick, capture);

		expect(sessao.extendEntity).toHaveBeenCalledWith(alvo, boundaries, pick.x, pick.y, capture);
	});

	it("filletEntities devolve o id quando a sessão devolve '9', e null quando a sessão devolve undefined", () => {
		const sessao = sessaoFalsa({
			filletEntities: vi.fn(() => '9')
		});
		const doc = documentoSobre(sessao);
		const a = id('1');
		const pickA = { x: 1, y: 1 };
		const b = id('2');
		const pickB = { x: 2, y: 2 };
		const radius = 2;

		const result = doc.filletEntities(a, pickA, b, pickB, radius);

		expect(result).toBe('9');
		expect(sessao.filletEntities).toHaveBeenCalledWith(
			a,
			pickA.x,
			pickA.y,
			b,
			pickB.x,
			pickB.y,
			radius
		);

		sessao.filletEntities.mockImplementation(() => undefined);
		const nullResult = doc.filletEntities(a, pickA, b, pickB, radius);
		expect(nullResult).toBeNull();
	});

	it('chamferEntities repassa as duas distâncias (2 e 3) e devolve o id', () => {
		const sessao = sessaoFalsa({
			chamferEntities: vi.fn(() => '11')
		});
		const doc = documentoSobre(sessao);
		const a = id('1');
		const pickA = { x: 1, y: 1 };
		const b = id('2');
		const pickB = { x: 2, y: 2 };
		const d1 = 2;
		const d2 = 3;

		const result = doc.chamferEntities(a, pickA, b, pickB, d1, d2);

		expect(result).toBe('11');
		expect(sessao.chamferEntities).toHaveBeenCalledWith(
			a,
			pickA.x,
			pickA.y,
			b,
			pickB.x,
			pickB.y,
			d1,
			d2
		);
	});

	it('findIntersections converte [{x:5,y:5}] em [{x:5,y:5}]; findEntitiesInBox repassa a caixa achatada e devolve os ids; hitTest repassa ponto e raio', () => {
		const sessao = sessaoFalsa({
			intersections: vi.fn(() => [{ x: 5, y: 5 }]),
			entitiesInBox: vi.fn(() => ['1', '2']),
			hitTest: vi.fn(() => ['3'])
		});
		const doc = documentoSobre(sessao);
		const a = id('1');
		const b = id('2');
		const area = { minX: 0, minY: 0, maxX: 10, maxY: 10 };
		const point = { x: 5, y: 5 };
		const radius = 2;

		const intersections = doc.findIntersections(a, b);
		const entitiesInBox = doc.findEntitiesInBox(area);
		const hitTestResult = doc.hitTest(point, radius);

		expect(intersections).toEqual([{ x: 5, y: 5 }]);
		expect(sessao.intersections).toHaveBeenCalledWith(a, b);
		expect(entitiesInBox).toEqual(['1', '2']);
		expect(sessao.entitiesInBox).toHaveBeenCalledWith(area.minX, area.minY, area.maxX, area.maxY);
		expect(hitTestResult).toEqual(['3']);
		expect(sessao.hitTest).toHaveBeenCalledWith(point.x, point.y, radius);
	});

	it("snap converte {mode:'endpoint', point:{x:0,y:0}, entity:'3'} em {mode:'endpoint', point:{x:0,y:0}, entityId:'3'} e repassa os modos; snap devolve null quando a sessão devolve null", () => {
		const sessao = sessaoFalsa({
			snap: vi.fn(() => ({ mode: 'endpoint', point: { x: 0, y: 0 }, entity: '3' }))
		});
		const doc = documentoSobre(sessao);
		const point = { x: 1, y: 1 };
		const radius = 2;
		const modes: CadSnapMode[] = ['endpoint', 'midpoint'];

		const result = doc.snap(point, radius, modes);

		expect(result).toEqual({ mode: 'endpoint', point: { x: 0, y: 0 }, entityId: '3' });
		expect(sessao.snap).toHaveBeenCalledWith(point.x, point.y, radius, modes);

		sessao.snap.mockImplementation(() => null);
		const nullResult = doc.snap(point, radius, modes);
		expect(nullResult).toBeNull();
	});

	it('undo e redo devolvem false quando a sessão devolve false, sem lançar; e true quando a sessão devolve true', () => {
		const sessao = sessaoFalsa({
			undo: vi.fn(() => false),
			redo: vi.fn(() => true)
		});
		const doc = documentoSobre(sessao);

		expect(doc.undo()).toBe(false);
		expect(doc.redo()).toBe(true);
		expect(sessao.undo).toHaveBeenCalled();
		expect(sessao.redo).toHaveBeenCalled();
	});
});

describe('CadDocument: erros (W3, W4, W4b)', () => {
	it('recusa do kernel em moveEntities vira CadKernelOperationError com kernelMessage "a operação não pôde ser feita: fator inválido", e isTrusted continua true', () => {
		const sessao = sessaoFalsa({
			moveEntities: vi.fn(() => {
				throw new Error('a operação não pôde ser feita: fator inválido');
			})
		});
		const doc = documentoSobre(sessao);

		try {
			doc.moveEntities([id('1')], { x: 10, y: 20 });
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelOperationError);
			expect((e as CadKernelOperationError).kernelMessage).toBe(
				'a operação não pôde ser feita: fator inválido'
			);
			expect(doc.isTrusted).toBe(true);
		}

		expect(sessao.moveEntities).toHaveBeenCalledWith([id('1')], 10, 20);
	});

	it('recusa do kernel em filletEntities também vira CadKernelOperationError (não Error genérico)', () => {
		const sessao = sessaoFalsa({
			filletEntities: vi.fn(() => {
				throw new Error('erro de cálculo');
			})
		});
		const doc = documentoSobre(sessao);

		try {
			doc.filletEntities(id('1'), { x: 0, y: 0 }, id('2'), { x: 1, y: 1 }, 5);
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelOperationError);
			expect((e as CadKernelOperationError).kernelMessage).toBe('erro de cálculo');
		}

		expect(sessao.filletEntities).toHaveBeenCalledWith(id('1'), 0, 0, id('2'), 1, 1, 5);
	});

	it('findIntersections com resposta [{x:1}] lança CadKernelContractError cuja mensagem contém "interseção[0].y"', () => {
		const sessao = sessaoFalsa({
			intersections: vi.fn(() => [{ x: 1 }])
		});
		const doc = documentoSobre(sessao);

		try {
			doc.findIntersections(id('1'), id('2'));
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelContractError);
			expect((e as Error).message).toContain('interseção[0].y');
		}

		expect(sessao.intersections).toHaveBeenCalledWith(id('1'), id('2'));
	});

	it('snap com resposta {mode:"xyz", point:{x:0,y:0}, entity:"3"} lança CadKernelContractError cuja mensagem contém "prendimento.mode"', () => {
		const sessao = sessaoFalsa({
			snap: vi.fn(() => ({ mode: 'xyz', point: { x: 0, y: 0 }, entity: '3' }))
		});
		const doc = documentoSobre(sessao);

		try {
			doc.snap({ x: 10, y: 20 }, 5, ['endpoint']);
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelContractError);
			expect((e as Error).message).toContain('prendimento.mode');
		}

		expect(sessao.snap).toHaveBeenCalledWith(10, 20, 5, ['endpoint']);
	});

	it('hitTest com resposta [42] (número, não texto) lança CadKernelContractError cuja mensagem contém "ids[0]"', () => {
		const sessao = sessaoFalsa({
			hitTest: vi.fn(() => [42])
		});
		const doc = documentoSobre(sessao);

		try {
			doc.hitTest({ x: 10, y: 20 }, 5);
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelContractError);
			expect((e as Error).message).toContain('ids[0]');
		}

		expect(sessao.hitTest).toHaveBeenCalledWith(10, 20, 5);
	});

	it('RuntimeError em trimEntity lança CadKernelFailureError e isTrusted vira false', () => {
		const sessao = sessaoFalsa({
			trimEntity: vi.fn(() => {
				throw new WebAssembly.RuntimeError('unreachable');
			})
		});
		const doc = documentoSobre(sessao);

		try {
			doc.trimEntity(id('1'), [], { x: 0, y: 0 }, 0.5);
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelFailureError);
			expect(doc.isTrusted).toBe(false);
		}

		expect(sessao.trimEntity).toHaveBeenCalledWith(id('1'), [], 0, 0, 0.5);
	});

	it('Depois do RuntimeError, listEntities(), undo() e moveEntities(...) lançam CadKernelFailureError e a sessão NÃO é chamada', () => {
		const sessao = sessaoFalsa({
			trimEntity: vi.fn(() => {
				throw new WebAssembly.RuntimeError('unreachable');
			}),
			entities: vi.fn(() => []),
			undo: vi.fn(() => false),
			moveEntities: vi.fn()
		});
		const doc = documentoSobre(sessao);

		// Primeiro, causar o erro
		try {
			doc.trimEntity(id('1'), [], { x: 0, y: 0 }, 0.5);
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelFailureError);
		}

		// Agora testar que os métodos falham e não chamam a sessão
		expect(() => doc.listEntities()).toThrow(CadKernelFailureError);
		expect(() => doc.undo()).toThrow(CadKernelFailureError);
		expect(() => doc.moveEntities([id('1')], { x: 10, y: 20 })).toThrow(CadKernelFailureError);

		// Verificar que os métodos não foram chamados
		expect(sessao.entities).not.toHaveBeenCalled();
		expect(sessao.undo).not.toHaveBeenCalled();
		expect(sessao.moveEntities).not.toHaveBeenCalled();
	});

	it('Depois do RuntimeError, toDxf() ainda chama a sessão e devolve os bytes', () => {
		const sessao = sessaoFalsa({
			trimEntity: vi.fn(() => {
				throw new WebAssembly.RuntimeError('unreachable');
			}),
			toDxf: vi.fn(() => new Uint8Array([1, 2, 3]))
		});
		const doc = documentoSobre(sessao);

		// Primeiro, causar o erro
		try {
			doc.trimEntity(id('1'), [], { x: 0, y: 0 }, 0.5);
			expect.unreachable();
		} catch (e) {
			expect(e).toBeInstanceOf(CadKernelFailureError);
		}

		// toDxf ainda deve funcionar e chamar a sessão
		const result = doc.toDxf();
		expect(result).toEqual(new Uint8Array([1, 2, 3]));
		expect(sessao.toDxf).toHaveBeenCalled();
	});
});

// Rascunho do qwen3-coder:30b a partir do critério de aceite (MT-KV-06); o teste da
// consulta foi corrigido para não supor a forma de CadEntity e conferir os ids repassados.
describe('CadDocument: o que mudou e consulta pelo id (MT-KV-06)', () => {
	it('lastChange chama sessao.lastChange e converte', () => {
		const sessao = sessaoFalsa({
			lastChange: vi.fn(() => ({ created: ['5'], modified: [], removed: ['6'], layers: [] }))
		});
		const doc = documentoSobre(sessao);
		expect(doc.lastChange()).toEqual({ created: ['5'], modified: [], removed: ['6'], layers: [] });
	});

	it('lastChange lança CadKernelContractError quando sessão devolve objeto inválido', () => {
		const sessao = sessaoFalsa({
			lastChange: vi.fn(() => ({ created: 5 }))
		});
		const doc = documentoSobre(sessao);
		expect(() => doc.lastChange()).toThrow(CadKernelContractError);
	});

	it('getEntitiesById chama sessao.entitiesById e converte', () => {
		const ENTIDADE = {
			id: '1',
			layer: '7',
			geometry: { kind: 'line', start: { x: 0, y: 0 }, end: { x: 3, y: 4 } },
			bounds: { minX: 0, minY: 0, maxX: 3, maxY: 4 }
		};
		const sessao = sessaoFalsa({
			entitiesById: vi.fn(() => [ENTIDADE, null])
		});
		const doc = documentoSobre(sessao);
		const achadas = doc.getEntitiesById([id('1'), id('2')]);

		expect(sessao.entitiesById).toHaveBeenCalledWith(['1', '2']);
		expect(achadas).toHaveLength(2);
		expect(achadas[0]?.id).toBe('1');
		expect(achadas[0]?.geometry.kind).toBe('line');
		expect(achadas[1]).toBeNull();
	});

	it('getEntitiesById lança CadKernelOperationError quando sessão lança erro', () => {
		const sessao = sessaoFalsa({
			entitiesById: vi.fn(() => {
				throw new Error('identificador de entidade inválido');
			})
		});
		const doc = documentoSobre(sessao);
		expect(() => doc.getEntitiesById([id('x')])).toThrow(CadKernelOperationError);
	});
});

describe('CadDocument: geometria da tela (MT-KV-08)', () => {
	it('prepareScreen repassa o erro de corda', () => {
		const sessao = sessaoFalsa();

		documentoSobre(sessao).prepareScreen(0.25);

		expect(sessao.prepareScreen).toHaveBeenCalledWith(0.25);
	});

	it('prepareScreen recusado vira CadKernelOperationError', () => {
		const sessao = sessaoFalsa({
			prepareScreen: vi.fn(() => {
				throw new Error('erro de corda inválido: 0');
			})
		});

		expect(() => documentoSobre(sessao).prepareScreen(0)).toThrow(CadKernelOperationError);
	});

	it('espaços, origem e camadas vêm do kernel', () => {
		const sessao = sessaoFalsa({
			screenSpaces: vi.fn(() => ['Model', 'Prancha A1']),
			screenOrigin: vi.fn(() => new Float64Array([5_000_000.5, 7_500_000.25])),
			screenLayers: vi.fn(() => ['3', '9'])
		});
		const doc = documentoSobre(sessao);

		expect(doc.getScreenSpaces()).toEqual(['Model', 'Prancha A1']);
		expect(doc.getScreenOrigin('Prancha A1')).toEqual({ x: 5_000_000.5, y: 7_500_000.25 });
		expect(sessao.screenOrigin).toHaveBeenCalledWith('Prancha A1');
		expect(doc.getScreenLayers('Model')).toEqual(['3', '9']);
		expect(sessao.screenLayers).toHaveBeenCalledWith('Model');
	});

	it('origem que não tem dois números é erro de contrato', () => {
		const sessao = sessaoFalsa({ screenOrigin: vi.fn(() => new Float64Array([1])) });

		expect(() => documentoSobre(sessao).getScreenOrigin('Model')).toThrow(CadKernelContractError);
	});

	it('o bloco é o mesmo Float32Array do kernel, sem cópia', () => {
		const bloco = new Float32Array([0, 0, 1, 1, 2, 2, 3, 3]);
		const sessao = sessaoFalsa({ screenBlock: vi.fn(() => bloco) });

		const recebido = documentoSobre(sessao).getScreenBlock('Model', '3' as CadLayerId);

		expect(recebido).toBe(bloco);
		expect(sessao.screenBlock).toHaveBeenCalledWith('Model', '3');
	});

	it('bloco que não é Float32Array de segmentos inteiros é erro de contrato', () => {
		for (const ruim of [[0, 0, 1, 1], new Float32Array([0, 0, 1])]) {
			const sessao = sessaoFalsa({ screenBlock: vi.fn(() => ruim) });

			expect(() => documentoSobre(sessao).getScreenBlock('Model', '3' as CadLayerId)).toThrow(
				CadKernelContractError
			);
		}
	});

	it('updateScreen converte os blocos que mudaram', () => {
		const sessao = sessaoFalsa({
			updateScreen: vi.fn(() => [{ space: 'Model', layer: '3' }])
		});

		expect(documentoSobre(sessao).updateScreen()).toEqual([{ space: 'Model', layer: '3' }]);
	});

	it('updateScreen com bloco malformado é erro de contrato', () => {
		const sessao = sessaoFalsa({ updateScreen: vi.fn(() => [{ space: 'Model' }]) });

		expect(() => documentoSobre(sessao).updateScreen()).toThrow(CadKernelContractError);
	});

	it('findEntitiesInView repassa espaço e área', () => {
		const sessao = sessaoFalsa({ screenQuery: vi.fn(() => ['7']) });

		const achadas = documentoSobre(sessao).findEntitiesInView('Model', {
			minX: -1,
			minY: -2,
			maxX: 3,
			maxY: 4
		});

		expect(achadas).toEqual(['7']);
		expect(sessao.screenQuery).toHaveBeenCalledWith('Model', -1, -2, 3, 4);
	});
});
