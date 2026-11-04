// Caminho relativo: src/lib/services/preferencias.spec.ts

/**
 * \file src/lib/services/preferencias.spec.ts
 * \brief O modo decimal lembrado entre sessões (spec 10, D10).
 * \author Iago Leal
 * \date 2026-11-04
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { lerModoDecimal, gravarModoDecimal, CHAVE_MODO_DECIMAL } from './preferencias';

type LocalStorageMock = {
	getItem: (key: string) => string | null;
	setItem: (key: string, value: string) => void;
	removeItem: (key: string) => void;
	clear: () => void;
};

function createLocalStorageMock(): LocalStorageMock {
	const store = new Map<string, string>();
	return {
		getItem: (key: string) => store.get(key) ?? null,
		setItem: (key: string, value: string) => {
			store.set(key, value);
		},
		removeItem: (key: string) => {
			store.delete(key);
		},
		clear: () => {
			store.clear();
		}
	};
}

const originalWindowDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'window');

describe('preferencias - D10', () => {
	beforeEach(() => {
		Object.defineProperty(globalThis, 'window', {
			configurable: true,
			value: { localStorage: createLocalStorageMock() }
		});
	});

	afterEach(() => {
		if (originalWindowDescriptor) {
			Object.defineProperty(globalThis, 'window', originalWindowDescriptor);
		} else {
			Reflect.deleteProperty(globalThis, 'window');
		}
	});

	it('sem nada gravado, lerModoDecimal retorna ponto', () => {
		expect(lerModoDecimal()).toBe('ponto');
	});

	it('gravar e ler modo virgula e ponto', () => {
		gravarModoDecimal('virgula');
		expect(lerModoDecimal()).toBe('virgula');
		gravarModoDecimal('ponto');
		expect(lerModoDecimal()).toBe('ponto');
	});

	it('valor estranho gravado retorna ponto', () => {
		window.localStorage.setItem(CHAVE_MODO_DECIMAL, 'xyz');
		expect(lerModoDecimal()).toBe('ponto');
	});

	it('se localStorage.getItem lança erro, lerModoDecimal retorna ponto', () => {
		Object.defineProperty(window, 'localStorage', {
			configurable: true,
			value: {
				...createLocalStorageMock(),
				getItem: () => {
					throw new Error('fail');
				}
			}
		});
		expect(lerModoDecimal()).toBe('ponto');
	});

	it('se localStorage.setItem lança erro, gravarModoDecimal não lança', () => {
		Object.defineProperty(window, 'localStorage', {
			configurable: true,
			value: {
				...createLocalStorageMock(),
				setItem: () => {
					throw new Error('fail');
				}
			}
		});
		expect(() => gravarModoDecimal('virgula')).not.toThrow();
	});

	it('sem window, lerModoDecimal retorna ponto e gravar não lança', () => {
		Reflect.deleteProperty(globalThis, 'window');
		expect(lerModoDecimal()).toBe('ponto');
		expect(() => gravarModoDecimal('virgula')).not.toThrow();
	});
});
