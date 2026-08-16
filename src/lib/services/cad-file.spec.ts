// Caminho relativo: src/lib/services/cad-file.spec.ts

/**
 * \file src/lib/services/cad-file.spec.ts
 * \brief Testes unitários para serviços de seleção e metadados de arquivos CAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { describe, expect, it } from 'vitest';
import {
	createCadDocumentPayloadFromFile,
	extractCadFileName,
	isSupportedCadFile,
	toDxfFileName
} from './cad-file';

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

	it('normaliza um File em payload compatível com o viewer', async () => {
		const file = new File([new Uint8Array([1, 2, 3])], 'drag-drop-sample.dwg');
		const payload = await createCadDocumentPayloadFromFile(file);

		expect(payload.fileName).toBe('drag-drop-sample.dwg');
		expect(payload.source).toBe('browser');
		expect(payload.content.byteLength).toBe(3);
	});
});

describe('nome do arquivo gravado', () => {
	it('troca a extensão para dxf', () => {
		// Escrita DWG depende de especificação fechada e fica fora do projeto;
		// manter a extensão original faria o arquivo mentir sobre o conteúdo.
		expect(toDxfFileName('planta.dwg')).toBe('planta.dxf');
		expect(toDxfFileName('planta.DXF')).toBe('planta.dxf');
	});

	it('acrescenta a extensão quando não há nenhuma', () => {
		expect(toDxfFileName('planta')).toBe('planta.dxf');
	});

	it('preserva ponto no meio do nome', () => {
		expect(toDxfFileName('LT-138kV.rev2.dwg')).toBe('LT-138kV.rev2.dxf');
	});

	it('não confunde ponto de diretório com extensão', () => {
		expect(toDxfFileName('/home/iago/proj.v1/planta')).toBe('/home/iago/proj.v1/planta.dxf');
	});
});
