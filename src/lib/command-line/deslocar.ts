// Caminho relativo: src/lib/command-line/deslocar.ts

/**
 * \file src/lib/command-line/deslocar.ts
 * \brief O comando OFFSET / DESLOCAMENTO / O (spec 10, O1–O4).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Distância, objeto escolhido pelo clique na tela (sem Enter, como no AutoCAD) e
 * lado digitado; repete até Sair. Cada paralela é um passo de desfazer.
 */

import type { CadEntityId } from '$lib/types/cad';
import { lerOpcao, type ComandoEmCurso, type ContextoDeComando, type Opcao } from './motor';
import { lerNumero } from './pontos';
import { ehPonto, pontoOuErro } from './pontos-do-prompt';

const SAIR: Opcao = { pt: 'Sair', abreviacaoPt: 'S', en: 'Exit', abreviacaoEn: 'E' };
const DESFAZER: Opcao = { pt: 'Desfazer', abreviacaoPt: 'D', en: 'Undo', abreviacaoEn: 'U' };

/** As paralelas de um lado, e a altura da pilha logo depois delas. */
type Paralela = { ids: CadEntityId[]; profundidade: number };

export function deslocar({ documento, aposEditar, selecao }: ContextoDeComando): ComandoEmCurso {
	let distancia: number | null = null;
	let objeto: CadEntityId | null = null;
	const paralelas: Paralela[] = [];

	/** Desfaz a última paralela; se outra ação entrou depois, apaga-a. */
	function desfazerUltima(): void {
		const paralela = paralelas.pop();
		if (paralela == null) return;
		if (documento.getHistory().undoDepth === paralela.profundidade) documento.undo();
		else documento.eraseEntities(paralela.ids);
		aposEditar();
	}

	return {
		prompt() {
			if (distancia == null) return 'Especifique a distância de deslocamento:';
			if (objeto == null) return 'Selecione o objeto a deslocar ou [Sair/Desfazer] <Sair>:';
			return 'Especifique o ponto no lado a deslocar:';
		},

		aoSelecionar(ids) {
			const ultimo = ids.at(-1);
			if (distancia != null && objeto == null && ultimo != null) objeto = ultimo;
			return { tipo: 'continua' };
		},

		responder(texto) {
			if (distancia == null) {
				if (texto.trim() === '') return { tipo: 'fim' };
				const valor = lerNumero(texto);
				if (valor == null || !(valor > 0)) {
					return { tipo: 'invalida', mensagem: 'Esperava uma distância maior que zero.' };
				}
				distancia = valor;
				return { tipo: 'continua' };
			}

			if (objeto == null) {
				if (texto.trim() === '') return { tipo: 'fim' };
				const opcao = lerOpcao(texto, [SAIR, DESFAZER]);
				if (opcao === SAIR) return { tipo: 'fim' };
				if (opcao === DESFAZER) {
					if (paralelas.length === 0) {
						return { tipo: 'invalida', mensagem: 'Não há paralela para desfazer.' };
					}
					desfazerUltima();
					return { tipo: 'continua' };
				}
				return { tipo: 'invalida', mensagem: 'Clique no objeto a deslocar, na tela.' };
			}

			if (texto.trim() === '') return { tipo: 'fim' };
			const lado = pontoOuErro(texto, null);
			if (!ehPonto(lado)) return lado;

			// O objeto sai antes da chamada: se o kernel recusar, o motor leva o motivo
			// ao histórico e a linha volta a pedir objeto (O3).
			const escolhido = objeto;
			objeto = null;
			selecao.esvaziar();
			const ids = documento.offsetEntity(escolhido, distancia, lado);
			paralelas.push({ ids, profundidade: documento.getHistory().undoDepth });
			aposEditar();
			return { tipo: 'continua' };
		},

		encerrar() {
			selecao.esvaziar();
		}
	};
}
