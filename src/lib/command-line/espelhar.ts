// Caminho relativo: src/lib/command-line/espelhar.ts

/**
 * \file src/lib/command-line/espelhar.ts
 * \brief O comando MIRROR / ESPELHAR / MI (spec 10, R2 e R4).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Por padrão, as originais ficam e entram cópias espelhadas, como no AutoCAD; o
 * kernel faz as duas coisas numa só edição.
 */

import type { CadPoint } from '$lib/types/cad';
import { lerOpcao, type ComandoEmCurso, type ContextoDeComando, type Opcao } from './motor';
import { ehPonto, pontoOuErro } from './pontos-do-prompt';
import { sobreASelecao } from './selecao-etapa';

const SIM: Opcao = { pt: 'Sim', abreviacaoPt: 'S', en: 'Yes', abreviacaoEn: 'Y' };
const NAO: Opcao = { pt: 'Não', abreviacaoPt: 'N', en: 'No', abreviacaoEn: 'N' };

export function espelhar(contexto: ContextoDeComando): ComandoEmCurso {
	return sobreASelecao(contexto, (ids) => {
		let primeiro: CadPoint | null = null;
		let segundo: CadPoint | null = null;

		return {
			querPonto: () => segundo == null,
			prompt() {
				if (primeiro == null) return 'Especifique o primeiro ponto da linha de espelhamento:';
				if (segundo == null) return 'Especifique o segundo ponto da linha de espelhamento:';
				return 'Apagar objetos de origem? [Sim/Não] <Não>:';
			},
			responder(texto) {
				if (primeiro == null || segundo == null) {
					if (texto.trim() === '') return { tipo: 'fim' };
					const ponto = pontoOuErro(texto, primeiro);
					if (!ehPonto(ponto)) return ponto;
					if (primeiro == null) {
						primeiro = ponto;
						return { tipo: 'continua' };
					}
					if (ponto.x === primeiro.x && ponto.y === primeiro.y) {
						return {
							tipo: 'invalida',
							mensagem: 'O segundo ponto precisa ser diferente do primeiro.'
						};
					}
					segundo = ponto;
					return { tipo: 'continua' };
				}

				const opcao = texto.trim() === '' ? NAO : lerOpcao(texto, [SIM, NAO]);
				if (opcao == null) return { tipo: 'invalida', mensagem: 'Responda Sim ou Não.' };

				const angulo = Math.atan2(segundo.y - primeiro.y, segundo.x - primeiro.x);
				if (opcao === SIM) contexto.documento.mirrorEntities(ids, primeiro, angulo);
				else contexto.documento.mirrorCopyEntities(ids, primeiro, angulo);
				contexto.aposEditar();
				return { tipo: 'fim' };
			},
			encerrar: () => undefined
		};
	});
}
