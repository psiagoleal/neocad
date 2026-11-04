// Caminho relativo: src/lib/services/preferencias.ts

/**
 * \file src/lib/services/preferencias.ts
 * \brief Preferências lembradas entre sessões: por ora, o modo decimal (spec 10, D10).
 * \author Iago Leal
 * \date 2026-11-04
 */

import type { ModoDecimal } from '$lib/command-line/pontos';

export const CHAVE_MODO_DECIMAL = 'neocad.modo-decimal';

/** O modo gravado; sem gravação, ou se ela não puder ser lida, o padrão (`ponto`). */
export function lerModoDecimal(): ModoDecimal {
	try {
		return window.localStorage.getItem(CHAVE_MODO_DECIMAL) === 'virgula' ? 'virgula' : 'ponto';
	} catch {
		return 'ponto';
	}
}

/** Grava o modo; se não der (sem armazenamento), o modo vale só nesta sessão. */
export function gravarModoDecimal(modo: ModoDecimal): void {
	try {
		window.localStorage.setItem(CHAVE_MODO_DECIMAL, modo);
	} catch {
		// Sem onde gravar: segue sem lembrar.
	}
}
