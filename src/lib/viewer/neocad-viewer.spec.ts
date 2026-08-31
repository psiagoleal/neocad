// Caminho relativo: src/lib/viewer/neocad-viewer.spec.ts

/**
 * \file src/lib/viewer/neocad-viewer.spec.ts
 * \brief Testes unitários da montagem da lista de layouts.
 * \author Iago Leal
 * \date 2026-08-17
 *
 * Os objetos são sintéticos, no formato que o dicionário de layouts do upstream
 * produz (`AcDbLayout`). Isso é o que permite exercitar a lógica sem navegador —
 * o módulo só importa o upstream como tipo, nunca em tempo de execução.
 */

import { describe, expect, it } from 'vitest';
import { buildLayoutList, type CadLayoutSource } from './neocad-viewer';

/** Objeto no formato de um `AcDbLayout`. */
function layout(
	layoutName: string,
	blockTableRecordId: string,
	tabOrder = 0,
	viewportArray: string[] = []
) {
	return { layoutName, tabOrder, blockTableRecordId, viewportArray };
}

function fonte(
	layouts: unknown[],
	contagens: Record<string, number> = {},
	modelSpaceBlockId = 'bloco-modelo'
): CadLayoutSource {
	return {
		layouts,
		modelSpaceBlockId,
		entityCountOf: (blockId) => contagens[blockId] ?? null
	};
}

describe('lista de layouts', () => {
	it('lê nome, bloco, ordem e contagem de entidades', () => {
		const lista = buildLayoutList(
			fonte([layout('Prancha A1', 'bloco-papel', 1, ['vp-1', 'vp-2'])], {
				'bloco-papel': 19,
				'bloco-modelo': 484
			})
		);

		const prancha = lista.find((item) => item.name === 'Prancha A1');
		expect(prancha).toEqual({
			name: 'Prancha A1',
			blockId: 'bloco-papel',
			tabOrder: 1,
			entityCount: 19,
			isModelSpace: false,
			viewportCount: 2
		});
	});

	it('sintetiza o espaço-modelo quando o dicionário não o traz', () => {
		// Arquivo sem a aba `Model` ainda tem espaço-modelo. Sem este item, a
		// interface ficaria sem aba nenhuma que mostrasse o desenho.
		const lista = buildLayoutList(
			fonte([layout('Prancha', 'bloco-papel')], { 'bloco-modelo': 484 })
		);

		expect(lista).toHaveLength(2);
		expect(lista[0]).toMatchObject({
			name: 'Model',
			blockId: 'bloco-modelo',
			isModelSpace: true,
			entityCount: 484
		});
	});

	it('reconhece o espaço-modelo pelo bloco, e não pelo nome', () => {
		// O nome da aba é livre e pode estar traduzido; o bloco é a identidade.
		const lista = buildLayoutList(fonte([layout('Modelo', 'bloco-modelo')], { 'bloco-modelo': 3 }));

		expect(lista).toHaveLength(1);
		expect(lista[0]).toMatchObject({ name: 'Modelo', isModelSpace: true });
	});

	it('reconhece o espaço-modelo pelo nome quando o bloco não bate', () => {
		const lista = buildLayoutList(fonte([layout('Model', 'outro-bloco')]));

		expect(lista.filter((item) => item.isModelSpace)).toHaveLength(1);
		// E não sintetiza um segundo espaço-modelo por cima.
		expect(lista).toHaveLength(1);
	});

	it('põe o espaço-modelo primeiro e ordena o resto por aba', () => {
		const lista = buildLayoutList(
			fonte([
				layout('Prancha C', 'c', 3),
				layout('Model', 'bloco-modelo', 0),
				layout('Prancha A', 'a', 1),
				layout('Prancha B', 'b', 2)
			])
		);

		expect(lista.map((item) => item.name)).toEqual([
			'Model',
			'Prancha A',
			'Prancha B',
			'Prancha C'
		]);
	});

	it('desempata por nome para não depender da ordem do dicionário', () => {
		const comEmpate = [layout('Zebra', 'z', 1), layout('Alfa', 'a', 1)];

		expect(buildLayoutList(fonte(comEmpate)).map((item) => item.name)).toEqual(
			buildLayoutList(fonte([...comEmpate].reverse())).map((item) => item.name)
		);
	});

	it('ignora entrada sem nome ou sem bloco em vez de inventar layout', () => {
		const lista = buildLayoutList(
			fonte([
				layout('', 'bloco-papel'),
				{ layoutName: 'Sem bloco', tabOrder: 1 },
				null,
				'não é objeto',
				layout('Boa', 'bloco-boa')
			])
		);

		expect(lista.map((item) => item.name)).toEqual(['Model', 'Boa']);
	});

	it('bloco inexistente conta zero em vez de quebrar', () => {
		const lista = buildLayoutList(fonte([layout('Órfã', 'bloco-que-nao-existe')]));

		expect(lista.find((item) => item.name === 'Órfã')?.entityCount).toBe(0);
	});

	it('documento sem layout algum ainda entrega o espaço-modelo', () => {
		const lista = buildLayoutList(fonte([], { 'bloco-modelo': 0 }));

		expect(lista).toHaveLength(1);
		expect(lista[0].isModelSpace).toBe(true);
	});
});
