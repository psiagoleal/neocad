// Caminho relativo: src/lib/command-line/escalar.ts

/**
 * \file src/lib/command-line/escalar.ts
 * \brief O comando SCALE / ESCALA / SC (spec 10, R3 e R4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import type { CadPoint } from '$lib/types/cad';
import type { ComandoEmCurso, ContextoDeComando } from './motor';
import { PROMPT_DA_BASE } from './mover';
import { lerNumero } from './pontos';
import { ehPonto, pontoOuErro } from './pontos-do-prompt';
import { sobreASelecao } from './selecao-etapa';

export function escalar(contexto: ContextoDeComando): ComandoEmCurso {
	return sobreASelecao(contexto, (ids) => {
		let base: CadPoint | null = null;

		return {
			prompt: () => (base == null ? PROMPT_DA_BASE : 'Especifique o fator de escala:'),
			querPonto: () => base == null,
			responder(texto) {
				if (base == null) {
					if (texto.trim() === '') return { tipo: 'fim' };
					const ponto = pontoOuErro(texto, null);
					if (!ehPonto(ponto)) return ponto;
					base = ponto;
					return { tipo: 'continua' };
				}

				const fator = lerNumero(texto);
				if (fator == null || !(fator > 0)) {
					return { tipo: 'invalida', mensagem: 'Esperava um fator de escala maior que zero.' };
				}
				contexto.documento.scaleEntities(ids, base, fator);
				contexto.aposEditar();
				return { tipo: 'fim' };
			},
			encerrar: () => undefined
		};
	});
}
