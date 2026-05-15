/**
 * \file src/lib/services/recent-documents.ts
 * \brief Persistência de documentos recentes no frontend do NeoCAD.
 * \author Iago Leal
 * \date 2026-05-13
 */

import { isTauri } from '@tauri-apps/api/core';
import type { CadRecentDocument } from '$lib/types/cad';

const RECENT_DOCUMENTS_STORAGE_KEY = 'neocad.recentDocuments';
const RECENT_DOCUMENTS_STATE_DIR = 'state';
const RECENT_DOCUMENTS_STATE_FILE = `${RECENT_DOCUMENTS_STATE_DIR}/recent-documents.json`;
const MAX_RECENT_DOCUMENTS = 8;

function isBrowserRuntime(): boolean {
	return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

function normalizeRecentDocuments(value: unknown): CadRecentDocument[] {
	if (!Array.isArray(value)) {
		return [];
	}

	return value
		.flatMap((item) => {
			if (typeof item !== 'object' || item == null) {
				return [];
			}

			const record = item as Record<string, unknown>;
			const fileName = typeof record.fileName === 'string' ? record.fileName.trim() : '';
			const source: CadRecentDocument['source'] | null =
				record.source === 'browser' || record.source === 'tauri' ? record.source : null;
			const openedAt = typeof record.openedAt === 'string' ? record.openedAt.trim() : '';
			const path = typeof record.path === 'string' ? record.path.trim() : undefined;

			if (fileName.length === 0 || openedAt.length === 0 || source == null) {
				return [];
			}

			return [
				{
					fileName,
					source,
					openedAt,
					path: path && path.length > 0 ? path : undefined
				}
			];
		})
		.slice(0, MAX_RECENT_DOCUMENTS);
}

function loadRecentDocumentsFromBrowser(): CadRecentDocument[] {
	if (!isBrowserRuntime()) {
		return [];
	}

	const rawValue = window.localStorage.getItem(RECENT_DOCUMENTS_STORAGE_KEY);

	if (rawValue == null) {
		return [];
	}

	try {
		return normalizeRecentDocuments(JSON.parse(rawValue));
	} catch {
		return [];
	}
}

function persistRecentDocumentsToBrowser(documents: CadRecentDocument[]): void {
	if (!isBrowserRuntime()) {
		return;
	}

	window.localStorage.setItem(RECENT_DOCUMENTS_STORAGE_KEY, JSON.stringify(documents));
}

function clearRecentDocumentsFromBrowser(): void {
	if (!isBrowserRuntime()) {
		return;
	}

	window.localStorage.removeItem(RECENT_DOCUMENTS_STORAGE_KEY);
}

async function readRecentDocumentsFromTauri(): Promise<CadRecentDocument[] | null> {
	if (!isTauri()) {
		return null;
	}

	const [{ BaseDirectory }, { readTextFile }] = await Promise.all([
		import('@tauri-apps/api/path'),
		import('@tauri-apps/plugin-fs')
	]);

	try {
		const rawValue = await readTextFile(RECENT_DOCUMENTS_STATE_FILE, {
			baseDir: BaseDirectory.AppConfig
		});
		return normalizeRecentDocuments(JSON.parse(rawValue));
	} catch {
		return null;
	}
}

async function persistRecentDocumentsToTauri(documents: CadRecentDocument[]): Promise<void> {
	if (!isTauri()) {
		return;
	}

	const [{ BaseDirectory }, { mkdir, writeTextFile }] = await Promise.all([
		import('@tauri-apps/api/path'),
		import('@tauri-apps/plugin-fs')
	]);

	await mkdir(RECENT_DOCUMENTS_STATE_DIR, {
		baseDir: BaseDirectory.AppConfig,
		recursive: true
	});
	await writeTextFile(RECENT_DOCUMENTS_STATE_FILE, JSON.stringify(documents, null, 2), {
		baseDir: BaseDirectory.AppConfig
	});
}

function deduplicateRecentDocuments(
	currentDocuments: CadRecentDocument[],
	document: CadRecentDocument
): CadRecentDocument[] {
	const deduplicated = currentDocuments.filter((item) => {
		if (document.path && item.path) {
			return item.path !== document.path;
		}

		return item.fileName !== document.fileName || item.source !== document.source;
	});

	return [document, ...deduplicated].slice(0, MAX_RECENT_DOCUMENTS);
}

export async function listRecentDocuments(): Promise<CadRecentDocument[]> {
	const browserDocuments = loadRecentDocumentsFromBrowser();

	if (!isTauri()) {
		return browserDocuments;
	}

	const tauriDocuments = await readRecentDocumentsFromTauri();
	const resolvedDocuments = tauriDocuments ?? browserDocuments;

	persistRecentDocumentsToBrowser(resolvedDocuments);

	if (tauriDocuments == null && browserDocuments.length > 0) {
		void persistRecentDocumentsToTauri(browserDocuments);
	}

	return resolvedDocuments;
}

export async function registerRecentDocument(
	document: CadRecentDocument
): Promise<CadRecentDocument[]> {
	const currentDocuments = await listRecentDocuments();
	const nextDocuments = deduplicateRecentDocuments(currentDocuments, document);

	persistRecentDocumentsToBrowser(nextDocuments);
	await persistRecentDocumentsToTauri(nextDocuments);

	return nextDocuments;
}

export async function clearRecentDocuments(): Promise<void> {
	clearRecentDocumentsFromBrowser();
	await persistRecentDocumentsToTauri([]);
}

export function getRecentDocumentsStorageKey(): string {
	return RECENT_DOCUMENTS_STORAGE_KEY;
}

export function getRecentDocumentsStateFile(): string {
	return RECENT_DOCUMENTS_STATE_FILE;
}

export function getRecentDocumentsStateDirectory(): string {
	return RECENT_DOCUMENTS_STATE_DIR;
}

export function getMaxRecentDocuments(): number {
	return MAX_RECENT_DOCUMENTS;
}
