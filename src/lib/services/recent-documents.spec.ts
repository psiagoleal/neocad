/**
 * \file src/lib/services/recent-documents.spec.ts
 * \brief Testes unitários para a persistência de documentos recentes do NeoCAD.
 * \author Iago Leal
 * \date 2026-05-13
 */

import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { CadRecentDocument } from '$lib/types/cad';
import {
	clearRecentDocuments,
	getRecentDocumentsStorageKey,
	listRecentDocuments,
	registerRecentDocument
} from './recent-documents';

const originalWindowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');

function createLocalStorageMock(): Storage {
	const storage = new Map<string, string>();

	return {
		get length() {
			return storage.size;
		},
		clear() {
			storage.clear();
		},
		getItem(key: string) {
			return storage.has(key) ? (storage.get(key) ?? null) : null;
		},
		key(index: number) {
			return Array.from(storage.keys())[index] ?? null;
		},
		removeItem(key: string) {
			storage.delete(key);
		},
		setItem(key: string, value: string) {
			storage.set(key, value);
		}
	} as Storage;
}

function installBrowserWindow(): void {
	Object.defineProperty(globalThis, 'window', {
		value: { localStorage: createLocalStorageMock() },
		configurable: true,
		writable: true
	});
}

function restoreOriginalWindow(): void {
	if (originalWindowDescriptor) {
		Object.defineProperty(globalThis, 'window', originalWindowDescriptor);
		return;
	}

	Reflect.deleteProperty(globalThis, 'window');
}

function getStoredRecentDocumentsRaw(): string | null {
	const currentWindow = globalThis.window as Window & typeof globalThis & { localStorage: Storage };
	return currentWindow.localStorage.getItem(getRecentDocumentsStorageKey());
}

function createRecentDocument(overrides: Partial<CadRecentDocument> = {}): CadRecentDocument {
	return {
		fileName: 'sample.dwg',
		source: 'browser',
		openedAt: '2026-05-13T10:00:00.000Z',
		...overrides
	};
}

beforeEach(() => {
	installBrowserWindow();
});

afterEach(() => {
	restoreOriginalWindow();
});

describe('recent documents service', () => {
	it('persiste e lista documentos recentes no fallback de navegador', async () => {
		const document = createRecentDocument();

		const nextDocuments = await registerRecentDocument(document);
		const listedDocuments = await listRecentDocuments();

		expect(nextDocuments).toEqual([document]);
		expect(listedDocuments).toEqual([document]);
		expect(getStoredRecentDocumentsRaw()).toContain('sample.dwg');
	});

	it('desduplica documentos pelo caminho quando disponível', async () => {
		await registerRecentDocument(
			createRecentDocument({
				fileName: 'layout-base.dwg',
				source: 'tauri',
				path: 'C:/cad/layout-base.dwg',
				openedAt: '2026-05-13T10:00:00.000Z'
			})
		);

		const nextDocuments = await registerRecentDocument(
			createRecentDocument({
				fileName: 'layout-base.dwg',
				source: 'tauri',
				path: 'C:/cad/layout-base.dwg',
				openedAt: '2026-05-13T10:05:00.000Z'
			})
		);

		expect(nextDocuments).toHaveLength(1);
		expect(nextDocuments[0]?.openedAt).toBe('2026-05-13T10:05:00.000Z');
	});

	it('limpa a lista persistida de documentos recentes', async () => {
		await registerRecentDocument(createRecentDocument({ fileName: 'clear-me.dxf' }));

		await clearRecentDocuments();

		expect(await listRecentDocuments()).toEqual([]);
		expect(getStoredRecentDocumentsRaw()).toBeNull();
	});
});
