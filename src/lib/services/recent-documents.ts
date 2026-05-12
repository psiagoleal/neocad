// Caminho relativo: src/lib/services/recent-documents.ts

/**
 * \file src/lib/services/recent-documents.ts
 * \brief Persistência leve de documentos recentes para o frontend do NeoCAD.
 * \author Iago Leal
 * \date 2026-05-12
 */

import type { CadRecentDocument } from '$lib/types/cad';

const RECENT_DOCUMENTS_STORAGE_KEY = 'neocad.recentDocuments';
const MAX_RECENT_DOCUMENTS = 8;

function isBrowserRuntime(): boolean {
	return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
}

export function listRecentDocuments(): CadRecentDocument[] {
	if (!isBrowserRuntime()) {
		return [];
	}

	const rawValue = window.localStorage.getItem(RECENT_DOCUMENTS_STORAGE_KEY);

	if (rawValue == null) {
		return [];
	}

	try {
		const parsed = JSON.parse(rawValue) as CadRecentDocument[];
		return Array.isArray(parsed) ? parsed : [];
	} catch {
		return [];
	}
}

export function registerRecentDocument(document: CadRecentDocument): CadRecentDocument[] {
	if (!isBrowserRuntime()) {
		return [document];
	}

	const currentDocuments = listRecentDocuments();
	const deduplicated = currentDocuments.filter((item) => {
		if (document.path && item.path) {
			return item.path !== document.path;
		}

		return item.fileName !== document.fileName || item.source !== document.source;
	});

	const nextDocuments = [document, ...deduplicated].slice(0, MAX_RECENT_DOCUMENTS);
	window.localStorage.setItem(RECENT_DOCUMENTS_STORAGE_KEY, JSON.stringify(nextDocuments));
	return nextDocuments;
}

export function clearRecentDocuments(): void {
	if (!isBrowserRuntime()) {
		return;
	}

	window.localStorage.removeItem(RECENT_DOCUMENTS_STORAGE_KEY);
}
