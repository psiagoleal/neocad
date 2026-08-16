// Caminho relativo: src/lib/services/cad-file.ts

/**
 * \file src/lib/services/cad-file.ts
 * \brief Serviços para seleção, leitura e normalização de arquivos CAD em ambiente Tauri ou navegador.
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

export async function createCadDocumentPayloadFromFile(
	file: File,
	source: CadDocumentPayload['source'] = 'browser'
): Promise<CadDocumentPayload> {
	if (!isSupportedCadFile(file.name)) {
		throw new Error(`Formato de arquivo não suportado: ${file.name}`);
	}

	return {
		fileName: file.name,
		content: await file.arrayBuffer(),
		source
	};
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
					resolve(await createCadDocumentPayloadFromFile(file));
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

export async function readCadDocumentFromPath(path: string): Promise<CadDocumentPayload> {
	if (!isTauri()) {
		throw new Error('Leitura direta por caminho só está disponível no runtime Tauri.');
	}

	const fileName = extractCadFileName(path);

	if (!isSupportedCadFile(fileName)) {
		throw new Error(`Formato de arquivo não suportado: ${fileName}`);
	}

	const { readFile } = await import('@tauri-apps/plugin-fs');
	const bytes = await readFile(path);

	return {
		fileName,
		content: toArrayBuffer(bytes),
		source: 'tauri',
		path
	};
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

/**
 * Escolhe onde gravar o desenho.
 *
 * No Tauri, o diálogo de gravação **concede acesso ao arquivo escolhido** no
 * escopo do sistema de arquivos: é esse mecanismo que permite ao aplicativo
 * gravar sem receber permissão ampla de escrita. Fora dele, não há caminho a
 * escolher — o navegador entrega o arquivo por download.
 */
export async function chooseCadSavePath(defaultFileName: string): Promise<string | null> {
	if (!isTauri()) {
		return null;
	}

	const { save } = await import('@tauri-apps/plugin-dialog');

	const selectedPath = await save({
		title: 'Salvar desenho como',
		defaultPath: defaultFileName,
		filters: [{ name: 'Desenho DXF', extensions: ['dxf'] }]
	});

	return selectedPath ?? null;
}

/**
 * Grava os bytes do desenho.
 *
 * Com `path`, grava no arquivo (Tauri). Sem ele, entrega por download, que é o
 * que o navegador permite — e que, por não sobrescrever nada, nunca destrói
 * trabalho anterior.
 */
export async function writeCadDocument(
	bytes: Uint8Array,
	fileName: string,
	path?: string
): Promise<void> {
	if (isTauri() && path != null) {
		const { writeFile } = await import('@tauri-apps/plugin-fs');
		await writeFile(path, bytes);

		return;
	}

	downloadCadDocument(bytes, fileName);
}

/** Entrega o desenho por download do navegador. */
function downloadCadDocument(bytes: Uint8Array, fileName: string): void {
	const blob = new Blob([bytes as BlobPart], { type: 'application/dxf' });
	const url = URL.createObjectURL(blob);
	const anchor = document.createElement('a');

	anchor.href = url;
	anchor.download = fileName;
	anchor.style.display = 'none';
	document.body.append(anchor);
	anchor.click();
	anchor.remove();
	URL.revokeObjectURL(url);
}

/**
 * Troca a extensão do nome do arquivo para `.dxf`.
 *
 * Um desenho aberto de `.dwg` é gravado em DXF, porque escrita DWG depende de
 * especificação fechada e fica fora do projeto (ADR 0003). Manter a extensão
 * original faria o arquivo mentir sobre o próprio conteúdo.
 */
export function toDxfFileName(fileName: string): string {
	const semExtensao = fileName.replace(/\.[^./\\]+$/, '');

	return `${semExtensao || fileName}.dxf`;
}
