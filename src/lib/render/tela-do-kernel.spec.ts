// Caminho relativo: src/lib/render/tela-do-kernel.spec.ts

/**
 * \file src/lib/render/tela-do-kernel.spec.ts
 * \brief Aviso das entidades que a tela não exibe (spec 07, V6; MT-KV-13c).
 * \author Iago Leal
 * \date 2026-11-01
 *
 * Sem navegador: o canvas falso não oferece WebGL2, então a tela cai no canvas
 * 2D falso; o documento falso responde só o que a tela pergunta.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { CadDocument } from '$lib/services/cad-document';
import type { CadUnsupportedCount } from '$lib/types/cad';
import { TelaDoKernel } from './tela-do-kernel';

function canvasFalso(): HTMLCanvasElement {
	const ctx = {
		setTransform: () => undefined,
		fillRect: () => undefined,
		clearRect: () => undefined,
		beginPath: () => undefined,
		moveTo: () => undefined,
		lineTo: () => undefined,
		stroke: () => undefined
	};
	return {
		dataset: {},
		clientWidth: 800,
		clientHeight: 600,
		width: 0,
		height: 0,
		getContext: (tipo: string) => (tipo === '2d' ? ctx : null),
		addEventListener: () => undefined,
		removeEventListener: () => undefined,
		getBoundingClientRect: () => ({ left: 0, top: 0 })
	} as unknown as HTMLCanvasElement;
}

/** Documento com dois espaços; as não desenhadas de cada um podem mudar. */
function documentoFalso(recusadas: Record<string, CadUnsupportedCount[]>) {
	return {
		recusadas,
		prepareScreen: () => undefined,
		getBounds: () => null,
		getScreenSpaces: () => ['Model', 'Prancha A1'],
		getScreenOrigin: () => ({ x: 0, y: 0 }),
		getScreenLayers: () => [],
		getScreenExtents: () => null,
		getScreenEntityCount: () => 0,
		getScreenBlock: () => new Float32Array(),
		getScreenRefused: (espaco: string) => recusadas[espaco] ?? [],
		updateScreen: () => [],
		listLayers: () => []
	};
}

describe('tela do kernel: aviso das entidades não exibidas (V6)', () => {
	beforeEach(() => {
		vi.stubGlobal('requestAnimationFrame', () => 1);
		vi.stubGlobal('cancelAnimationFrame', () => undefined);
	});
	afterEach(() => vi.unstubAllGlobals());

	it('ao mostrar um espaço com entidades recusadas, avisa o tipo e a quantidade', () => {
		const doc = documentoFalso({
			'Prancha A1': [
				{ entityType: 'TEXT', count: 1 },
				{ entityType: 'VIEWPORT', count: 2 }
			]
		});
		const tela = new TelaDoKernel(canvasFalso(), { escuro: true });
		const avisos: string[] = [];
		tela.avisarCom((m) => avisos.push(m));

		tela.mostrar(doc as unknown as CadDocument);
		expect(avisos).toEqual([]);

		tela.mostrarEspaco('Prancha A1');
		expect(avisos).toEqual(['A tela não exibe 3 entidade(s) de Prancha A1: 1 TEXT, 2 VIEWPORT.']);
	});

	it('não repete o mesmo aviso, e avisa de novo quando a edição muda o que falta', () => {
		const doc = documentoFalso({ 'Prancha A1': [{ entityType: 'VIEWPORT', count: 1 }] });
		const tela = new TelaDoKernel(canvasFalso(), { escuro: true });
		const avisos: string[] = [];
		tela.avisarCom((m) => avisos.push(m));
		tela.mostrar(doc as unknown as CadDocument);

		tela.mostrarEspaco('Prancha A1');
		tela.mostrarEspaco('Model');
		tela.mostrarEspaco('Prancha A1');
		tela.atualizar();
		expect(avisos).toHaveLength(1);

		doc.recusadas['Prancha A1'] = [{ entityType: 'VIEWPORT', count: 2 }];
		tela.atualizar();
		expect(avisos).toEqual([
			'A tela não exibe 1 entidade(s) de Prancha A1: 1 VIEWPORT.',
			'A tela não exibe 2 entidade(s) de Prancha A1: 2 VIEWPORT.'
		]);
	});

	it('abrir outro desenho avisa de novo, mesmo com a mesma contagem', () => {
		const tela = new TelaDoKernel(canvasFalso(), { escuro: true });
		const avisos: string[] = [];
		tela.avisarCom((m) => avisos.push(m));
		const recusadas = { Model: [{ entityType: 'TEXT', count: 1 }] };

		tela.mostrar(documentoFalso(recusadas) as unknown as CadDocument);
		tela.mostrar(documentoFalso(recusadas) as unknown as CadDocument);
		expect(avisos).toHaveLength(2);
	});
});
