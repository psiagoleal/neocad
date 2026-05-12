// Caminho relativo: src/lib/services/cad-file.ts

/**
 * \file src/lib/services/cad-file.ts
 * \brief Serviços para seleção e leitura de arquivos CAD em ambiente Tauri ou navegador.
 * \author Iago Leal
 * \date 2026-05-12
 */

import { isTauri } from '@tauri-apps/api/core';
import type { CadDocumentPayload } from '$lib/types/cad';

const CAD_FILE_EXTENSIONS = ['dwg', 'dxf'] as const;

export function extractCadFileName(path: string): string {
	const normalizedPath = path.replaceAll('\\', '/');
	return normalizedPath.split('/').filter(Boolean).at(-1) ?? path;
}

export function isSupportedCadFile(fileName: string): boolean {
	const extension = fileName.split('.').pop()?.toLowerCase();
	return (
		extension != null &&
		CAD_FILE_EXTENSIONS.includes(extension as (typeof CAD_FILE_EXTENSIONS)[number])
	);
}

function toArrayBuffer(bytes: Uint8Array): ArrayBuffer {
	return bytes.slice().buffer;
}

async function openCadFileFromTauri(): Promise<CadDocumentPayload | null> {
	const [{ open }, { readFile }] = await Promise.all([
		import('@tauri-apps/plugin-dialog'),
		import('@tauri-apps/plugin-fs')
	]);

	const selectedPath = await open({
		title: 'Abrir desenho CAD',
		multiple: false,
		directory: false,
		filters: [
			{
				name: 'Arquivos CAD',
				extensions: [...CAD_FILE_EXTENSIONS]
			}
		]
	});

	if (selectedPath == null || Array.isArray(selectedPath)) {
		return null;
	}

	const fileName = extractCadFileName(selectedPath);
	const bytes = await readFile(selectedPath);

	return {
		fileName,
		content: toArrayBuffer(bytes),
		source: 'tauri',
		path: selectedPath
	};
}

function openCadFileFromBrowser(): Promise<CadDocumentPayload | null> {
	return new Promise((resolve, reject) => {
		const input = document.createElement('input');
		input.type = 'file';
		input.accept = '.dwg,.dxf';
		input.style.display = 'none';

		const cleanup = () => {
			input.remove();
		};

		input.addEventListener(
			'change',
			async () => {
				const file = input.files?.[0];
				cleanup();

				if (file == null) {
					resolve(null);
					return;
				}

				try {
					resolve({
						fileName: file.name,
						content: await file.arrayBuffer(),
						source: 'browser'
					});
				} catch (error) {
					reject(error);
				}
			},
			{ once: true }
		);

		document.body.append(input);
		input.click();
	});
}

export function getCadRuntimeLabel(): 'Tauri' | 'Browser' {
	return isTauri() ? 'Tauri' : 'Browser';
}

export async function selectCadDocument(): Promise<CadDocumentPayload | null> {
	const payload = isTauri() ? await openCadFileFromTauri() : await openCadFileFromBrowser();

	if (payload == null) {
		return null;
	}

	if (!isSupportedCadFile(payload.fileName)) {
		throw new Error(`Formato de arquivo não suportado: ${payload.fileName}`);
	}

	return payload;
}
