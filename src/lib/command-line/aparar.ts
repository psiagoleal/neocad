// Caminho relativo: src/lib/command-line/aparar.ts

/**
 * \file src/lib/command-line/aparar.ts
 * \brief Os comandos TRIM / APARAR / TR e EXTEND / ESTENDER / EX (spec 10, T1–T4).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Os dois têm a mesma forma: arestas pela seleção (o vazio sem seleção toma todas
 * as entidades do espaço), depois um objeto por clique, repetido até o vazio. O
 * clique vai ao comando pelo modo de escolha da tela, com o ponto clicado, que o
 * kernel usa para saber o trecho ou a ponta.
 */

import type { CadEntityId } from '$lib/types/cad';
import {
	lerOpcao,
	type ComandoEmCurso,
	type ContextoDeComando,
	type DocumentoDosComandos,
	type Escolha,
	type Opcao
} from './motor';

const DESFAZER: Opcao = { pt: 'Desfazer', abreviacaoPt: 'D', en: 'Undo', abreviacaoEn: 'U' };

type Forma = {
	/** "de corte" ou "de contorno". */
	arestas: string;
	/** "aparar" ou "estender". */
	verbo: string;
	operar(
		documento: DocumentoDosComandos,
		id: CadEntityId,
		arestas: CadEntityId[],
		e: Escolha
	): void;
};

function cortarOuEstender(forma: Forma) {
	return ({ documento, aposEditar, selecao }: ContextoDeComando): ComandoEmCurso => {
		let arestas: CadEntityId[] | null = null;
		/** Altura da pilha depois de cada corte ou extensão, para o Desfazer. */
		const feitos: number[] = [];

		return {
			prompt: () =>
				arestas == null
					? `Selecione arestas ${forma.arestas} ou <selecionar todos>:`
					: `Selecione o objeto a ${forma.verbo} ou [Desfazer]:`,

			querEscolha: () => arestas != null,

			aoEscolher(escolha) {
				const [maisPerto] = escolha.ids;
				if (arestas == null || maisPerto == null) {
					return { tipo: 'invalida', mensagem: 'Nenhum objeto sob o clique.' };
				}
				forma.operar(documento, maisPerto, arestas, escolha);
				feitos.push(documento.getHistory().undoDepth);
				aposEditar();
				return { tipo: 'continua' };
			},

			responder(texto) {
				if (arestas == null) {
					if (texto.trim() !== '') {
						return {
							tipo: 'invalida',
							mensagem: 'Selecione as arestas na tela; Enter segue (sem seleção, todas).'
						};
					}
					const selecionadas = selecao.ids();
					arestas = selecionadas.length > 0 ? selecionadas : selecao.todas();
					selecao.esvaziar();
					return { tipo: 'continua' };
				}

				if (texto.trim() === '') return { tipo: 'fim' };
				if (lerOpcao(texto, [DESFAZER]) === DESFAZER) {
					const ultimo = feitos.pop();
					if (ultimo == null) return { tipo: 'invalida', mensagem: 'Não há o que desfazer.' };
					// Só desfaz o que foi dele: se outra ação entrou depois, não mexe.
					if (documento.getHistory().undoDepth === ultimo) {
						documento.undo();
						aposEditar();
					}
					return { tipo: 'continua' };
				}
				return { tipo: 'invalida', mensagem: `Clique no objeto a ${forma.verbo}, na tela.` };
			},

			encerrar: () => undefined
		};
	};
}

export const aparar = cortarOuEstender({
	arestas: 'de corte',
	verbo: 'aparar',
	operar: (documento, id, arestas, { ponto, raio }) =>
		void documento.trimEntity(id, arestas, ponto, raio)
});

export const estender = cortarOuEstender({
	arestas: 'de contorno',
	verbo: 'estender',
	operar: (documento, id, arestas, { ponto, raio }) =>
		documento.extendEntity(id, arestas, ponto, raio)
});
