// Caminho relativo: src/lib/command-line/copiar.ts

/**
 * \file src/lib/command-line/copiar.ts
 * \brief O comando COPY / COPIAR / CO (spec 10, M3).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Como o MOVE, mas cada segundo ponto cria uma cópia, num passo de desfazer, e o
 * comando continua até o vazio ou `Sair`.
 */

import type { CadEntityId, CadPoint } from '$lib/types/cad';
import { lerOpcao, type ComandoEmCurso, type ContextoDeComando, type Opcao } from './motor';
import { PROMPT_DA_BASE, PROMPT_DO_SEGUNDO } from './mover';
import { ehPonto, pontoOuErro } from './pontos-do-prompt';
import { sobreASelecao } from './selecao-etapa';

const SAIR: Opcao = { pt: 'Sair', abreviacaoPt: 'S', en: 'Exit', abreviacaoEn: 'E' };
const DESFAZER: Opcao = { pt: 'Desfazer', abreviacaoPt: 'D', en: 'Undo', abreviacaoEn: 'U' };

/** As cópias de um segundo ponto, e a altura da pilha logo depois delas. */
type Copia = { ids: CadEntityId[]; profundidade: number };

export function copiar(contexto: ContextoDeComando): ComandoEmCurso {
	const { documento, aposEditar } = contexto;

	return sobreASelecao(contexto, (ids) => {
		let base: CadPoint | null = null;
		const copias: Copia[] = [];
		/** Depois da primeira cópia, mesmo desfeita, valem as opções Sair e Desfazer. */
		let comOpcoes = false;

		function copiarPara(delta: CadPoint): void {
			const novas = documento.copyEntities(ids, delta);
			copias.push({ ids: novas, profundidade: documento.getHistory().undoDepth });
			comOpcoes = true;
			aposEditar();
		}

		/** Desfaz a última cópia; se outra ação entrou depois, apaga as cópias dela. */
		function desfazerUltima(): void {
			const copia = copias.pop();
			if (copia == null) return;
			if (documento.getHistory().undoDepth === copia.profundidade) documento.undo();
			else documento.eraseEntities(copia.ids);
			aposEditar();
		}

		return {
			prompt() {
				if (base == null) return PROMPT_DA_BASE;
				return !comOpcoes
					? PROMPT_DO_SEGUNDO
					: 'Especifique o segundo ponto ou [Sair/Desfazer] <Sair>:';
			},
			responder(texto) {
				if (base == null) {
					if (texto.trim() === '') return { tipo: 'fim' };
					const ponto = pontoOuErro(texto, null);
					if (!ehPonto(ponto)) return ponto;
					base = ponto;
					return { tipo: 'continua' };
				}

				if (texto.trim() === '') {
					// Na primeira cópia, o vazio usa o ponto base como deslocamento.
					if (!comOpcoes) copiarPara(base);
					return { tipo: 'fim' };
				}

				if (comOpcoes) {
					const opcao = lerOpcao(texto, [SAIR, DESFAZER]);
					if (opcao === SAIR) return { tipo: 'fim' };
					if (opcao === DESFAZER) {
						if (copias.length === 0) {
							return { tipo: 'invalida', mensagem: 'Não há cópia para desfazer.' };
						}
						desfazerUltima();
						return { tipo: 'continua' };
					}
				}

				const destino = pontoOuErro(texto, base, comOpcoes ? [SAIR.pt, DESFAZER.pt] : []);
				if (!ehPonto(destino)) return destino;
				copiarPara({ x: destino.x - base.x, y: destino.y - base.y });
				return { tipo: 'continua' };
			},
			encerrar: () => undefined
		};
	});
}
