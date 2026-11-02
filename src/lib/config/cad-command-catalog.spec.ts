// Caminho relativo: src/lib/config/cad-command-catalog.spec.ts

/**
 * \file src/lib/config/cad-command-catalog.spec.ts
 * \brief O catálogo de `Ajuda > Comandos CAD` (spec 08, C11; spec 07, V5; ADR 0011).
 * \author Iago Leal
 * \date 2026-11-02
 */

import { describe, expect, it } from 'vitest';
import { CATALOGO_DA_SPEC } from '$lib/command-line/catalogo';
import { COMANDOS } from '$lib/command-line/comandos';
import { listCadCommandCatalog } from '$lib/services/cad-commands';
import { buildCadCommandCatalog } from './cad-command-catalog';

describe('catálogo de comandos (C11)', () => {
	const catalogo = listCadCommandCatalog();

	it('lista cada comando da tabela da spec 08, com inglês, português e apelidos', () => {
		expect(catalogo).toHaveLength(CATALOGO_DA_SPEC.length);
		const linha = catalogo.find((c) => c.command === 'LINE');
		expect(linha).toMatchObject({ id: 'LINE', ptbr: 'LINHA', aliases: ['L'], available: true });
		const hachura = catalogo.find((c) => c.command === 'HATCH');
		expect(hachura).toMatchObject({ ptbr: 'HACHURA', aliases: ['BH', 'H'], available: false });
	});

	it('só é disponível o que a linha de comando própria executa (V5)', () => {
		const disponiveis = catalogo.filter((c) => c.available).map((c) => c.command);
		expect(disponiveis.sort()).toEqual(Object.keys(COMANDOS).sort());
	});

	it('os disponíveis vêm primeiro; depois categoria e rótulo', () => {
		const lista = buildCadCommandCatalog(
			[
				{ en: 'ZOOM', ptbr: 'ZOOM', apelidos: ['Z'] },
				{ en: 'HATCH', ptbr: 'HACHURA', apelidos: ['H'] },
				{ en: 'LINE', ptbr: 'LINHA', apelidos: ['L'] },
				{ en: 'ERASE', ptbr: 'APAGAR', apelidos: ['E'] }
			],
			['LINE', 'ERASE']
		);
		expect(lista.map((c) => c.command)).toEqual(['LINE', 'ERASE', 'ZOOM', 'HATCH']);
		expect(lista.map((c) => c.category)).toEqual(['draw', 'modify', 'navigation', 'draw']);
	});

	it('o rótulo vem dos metadados de apresentação; sem eles, do nome em português', () => {
		const lista = buildCadCommandCatalog(
			[
				{ en: 'CIRCLE', ptbr: 'CIRCULO', apelidos: ['C'] },
				{ en: 'ALIGN', ptbr: 'ALINHAR', apelidos: ['AL'] }
			],
			[]
		);
		expect(lista).toHaveLength(2);
		const [conhecido, outro] = lista;
		expect(conhecido.label).toBe('Círculo');
		expect(outro).toMatchObject({ label: 'Alinhar', category: 'other' });
	});
});
