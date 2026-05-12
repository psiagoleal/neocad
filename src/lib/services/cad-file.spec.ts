// Caminho relativo: src/lib/services/cad-file.spec.ts

/**
 * \file src/lib/services/cad-file.spec.ts
 * \brief Testes unitários para serviços de seleção e metadados de arquivos CAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { describe, expect, it } from 'vitest';
import { extractCadFileName, isSupportedCadFile } from './cad-file';

describe('cad file helpers', () => {
	it('extrai o nome do arquivo a partir de caminhos windows e unix', () => {
		expect(extractCadFileName('C:\\cad\\sample.dwg')).toBe('sample.dwg');
		expect(extractCadFileName('/home/iago/sample.dxf')).toBe('sample.dxf');
	});

	it('aceita apenas extensões CAD suportadas no MVP', () => {
		expect(isSupportedCadFile('plant.dwg')).toBe(true);
		expect(isSupportedCadFile('layout.dxf')).toBe(true);
		expect(isSupportedCadFile('notes.txt')).toBe(false);
	});
});
