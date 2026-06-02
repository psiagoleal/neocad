/**
 * \file src/lib/config/cad-command-catalog.spec.ts
 * \brief Testes unitários da montagem do catálogo de comandos CAD.
 * \author Iago Leal
 * \date 2026-06-02
 */

import { describe, expect, it } from 'vitest';
import type { CadCommandDescriptor } from '$lib/types/cad';
import { buildCadCommandCatalog } from './cad-command-catalog';

function descriptor(overrides: Partial<CadCommandDescriptor> = {}): CadCommandDescriptor {
	return {
		globalName: 'LINE',
		localName: 'Linha',
		group: 'ACAD',
		...overrides
	};
}

describe('buildCadCommandCatalog', () => {
	it('aplica metadados de apresentação a comandos conhecidos', () => {
		const [item] = buildCadCommandCatalog([descriptor({ globalName: 'CIRCLE' })]);

		expect(item).toMatchObject({
			id: 'CIRCLE',
			command: 'CIRCLE',
			label: 'Círculo',
			category: 'draw',
			group: 'ACAD'
		});
	});

	it('usa fallback "other" e rótulo derivado para comandos desconhecidos', () => {
		const [item] = buildCadCommandCatalog([
			descriptor({ globalName: 'FOOBAR', localName: 'FOOBAR' })
		]);

		expect(item.category).toBe('other');
		expect(item.label).toBe('Foobar');
		expect(item.command).toBe('FOOBAR');
	});

	it('normaliza o id para maiúsculas preservando o comando original', () => {
		const [item] = buildCadCommandCatalog([descriptor({ globalName: 'zoom', localName: 'Zoom' })]);

		expect(item.id).toBe('ZOOM');
		expect(item.command).toBe('zoom');
		expect(item.category).toBe('navigation');
	});

	it('ordena por categoria (navegação→desenho→...) e depois por rótulo', () => {
		const catalog = buildCadCommandCatalog([
			descriptor({ globalName: 'ERASE', localName: 'Apagar' }),
			descriptor({ globalName: 'LINE', localName: 'Linha' }),
			descriptor({ globalName: 'CIRCLE', localName: 'Círculo' }),
			descriptor({ globalName: 'ZOOM', localName: 'Zoom' })
		]);

		expect(catalog.map((item) => item.command)).toEqual(['ZOOM', 'CIRCLE', 'LINE', 'ERASE']);
	});

	it('retorna catálogo vazio quando não há descritores', () => {
		expect(buildCadCommandCatalog([])).toEqual([]);
	});
});
