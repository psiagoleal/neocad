// Caminho relativo: src/lib/command-line/mover.ts

/**
 * \file src/lib/command-line/mover.ts
 * \brief O comando MOVE / MOVER / M (spec 10, M1, M2, M4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import type { CadPoint } from '$lib/types/cad';
import type { ComandoEmCurso, ContextoDeComando } from './motor';
import { ehPonto, pontoOuErro } from './pontos-do-prompt';
import { sobreASelecao } from './selecao-etapa';

export const PROMPT_DA_BASE = 'Especifique o ponto base:';
export const PROMPT_DO_SEGUNDO =
	'Especifique o segundo ponto ou <usar o primeiro ponto como deslocamento>:';

export function mover(contexto: ContextoDeComando): ComandoEmCurso {
	return sobreASelecao(contexto, (ids) => {
		let base: CadPoint | null = null;

		return {
			prompt: () => (base == null ? PROMPT_DA_BASE : PROMPT_DO_SEGUNDO),
			responder(texto) {
				if (base == null) {
					if (texto.trim() === '') return { tipo: 'fim' };
					const ponto = pontoOuErro(texto, null);
					if (!ehPonto(ponto)) return ponto;
					base = ponto;
					return { tipo: 'continua' };
				}

				// O vazio usa o ponto base como deslocamento (M2).
				const destino = texto.trim() === '' ? null : pontoOuErro(texto, base);
				if (destino != null && !ehPonto(destino)) return destino;
				const delta = destino == null ? base : { x: destino.x - base.x, y: destino.y - base.y };
				contexto.documento.moveEntities(ids, delta);
				contexto.aposEditar();
				return { tipo: 'fim' };
			},
			encerrar: () => undefined
		};
	});
}
