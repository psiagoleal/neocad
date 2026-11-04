// Caminho relativo: src/lib/services/sonda.spec.ts

/**
 * \file src/lib/services/sonda.spec.ts
 * \brief A sonda edita pelo documento e avisa a página depois de cada edição.
 * \author Iago Leal
 * \date 2026-11-01
 *
 * O documento é simulado: o `.wasm` não carrega no Node (ver
 * cad-document.spec.ts). Que a edição chega ao kernel e à tela, prova o
 * `kernel-view.e2e.ts`.
 */

import { describe, expect, it } from 'vitest';
import type { CadDocument } from '$lib/services/cad-document';
import type { CadEntityId, CadLayer, CadLayerId, CadPoint } from '$lib/types/cad';
import { criarSonda } from './sonda';

function camada(id: string, name: string): CadLayer {
	return {
		id: id as CadLayerId,
		name,
		color: { kind: 'index', index: 7 },
		isVisible: true,
		isOff: false,
		isFrozen: false,
		isLocked: false
	} as CadLayer;
}

function montar() {
	const chamadas: unknown[][] = [];
	const documento = {
		listLayers: () => [camada('1', 'Paredes'), camada('2', '0')],
		drawLine: (layer: CadLayerId, a: CadPoint, b: CadPoint) => {
			chamadas.push(['drawLine', layer, a, b]);
			return '9' as CadEntityId;
		},
		moveEntities: (ids: CadEntityId[], d: CadPoint) => chamadas.push(['moveEntities', ids, d]),
		eraseEntity: (id: CadEntityId) => chamadas.push(['eraseEntity', id])
	} as unknown as CadDocument;
	let avisos = 0;
	const sonda = criarSonda({ documento: () => documento, aposEditar: () => (avisos += 1) });
	return { sonda, chamadas, avisos: () => avisos };
}

describe('sonda de edição', () => {
	it('desenha a linha na camada 0, devolve o id e avisa a página', () => {
		const { sonda, chamadas, avisos } = montar();

		expect(sonda.desenharLinha(0, 0, 10, 5)).toBe('9');
		expect(chamadas).toEqual([['drawLine', '2', { x: 0, y: 0 }, { x: 10, y: 5 }]]);
		expect(avisos()).toBe(1);
	});

	it('move e apaga pelo documento, avisando a cada edição', () => {
		const { sonda, chamadas, avisos } = montar();

		sonda.mover(['9' as CadEntityId], 1, 2);
		sonda.apagar('9' as CadEntityId);

		expect(chamadas).toEqual([
			['moveEntities', ['9'], { x: 1, y: 2 }],
			['eraseEntity', '9']
		]);
		expect(avisos()).toBe(2);
	});

	it('sem documento aberto, recusa com mensagem', () => {
		const sonda = criarSonda({ documento: () => null, aposEditar: () => {} });

		expect(() => sonda.desenharLinha(0, 0, 1, 1)).toThrow('Nenhum desenho aberto');
		expect(() => sonda.apagar('x' as CadEntityId)).toThrow('Nenhum desenho aberto');
	});
});
