// Caminho relativo: src/lib/command-line/catalogo.spec.ts

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { CATALOGO_DA_SPEC } from './catalogo';
import { lerTabela } from './resolver';

describe('catálogo embutido no build', () => {
	it('é a tabela da spec 08, inteira', () => {
		const tsv = readFileSync(
			path.join(process.cwd(), 'docs/specs/08-comandos-autocad.tsv'),
			'utf-8'
		);
		expect(CATALOGO_DA_SPEC).toEqual(lerTabela(tsv));
		expect(CATALOGO_DA_SPEC.find((c) => c.en === 'LINE')?.ptbr).toBe('LINHA');
	});
});
