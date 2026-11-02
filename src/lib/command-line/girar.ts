// Caminho relativo: src/lib/command-line/girar.ts

/**
 * \file src/lib/command-line/girar.ts
 * \brief O comando ROTATE / ROTAC / RO (spec 10, R1 e R4).
 * \author Iago Leal
 * \date 2026-11-02
 */

import type { CadPoint } from '$lib/types/cad';
import type { ComandoEmCurso, ContextoDeComando } from './motor';
import { lerNumero, lerPonto } from './pontos';
import { PROMPT_DA_BASE } from './mover';
import { ehPonto, pontoOuErro } from './pontos-do-prompt';
import { sobreASelecao } from './selecao-etapa';

export function girar(contexto: ContextoDeComando): ComandoEmCurso {
	return sobreASelecao(contexto, (ids) => {
		let base: CadPoint | null = null;

		return {
			prompt: () => (base == null ? PROMPT_DA_BASE : 'Especifique o ângulo de rotação:'),
			responder(texto) {
				if (base == null) {
					if (texto.trim() === '') return { tipo: 'fim' };
					const ponto = pontoOuErro(texto, null);
					if (!ehPonto(ponto)) return ponto;
					base = ponto;
					return { tipo: 'continua' };
				}

				// Graus, anti-horário; ou um ponto, valendo a direção da base até ele.
				let radianos: number | null = null;
				const graus = lerNumero(texto);
				if (graus != null) {
					radianos = (graus * Math.PI) / 180;
				} else {
					const leitura = lerPonto(texto, base);
					if (leitura.tipo === 'ponto') {
						radianos = Math.atan2(leitura.ponto.y - base.y, leitura.ponto.x - base.x);
					}
				}
				if (radianos == null) {
					return { tipo: 'invalida', mensagem: 'Esperava um ângulo em graus ou um ponto.' };
				}

				contexto.documento.rotateEntities(ids, base, radianos);
				contexto.aposEditar();
				return { tipo: 'fim' };
			},
			encerrar: () => undefined
		};
	});
}
