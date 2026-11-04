// Caminho relativo: src/lib/command-line/linha.ts

/**
 * \file src/lib/command-line/linha.ts
 * \brief O comando LINE / LINHA / L (spec 10, D1–D6).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Cada ponto depois do primeiro cria uma reta pelo kernel, na camada corrente.
 * Cada reta é um passo de desfazer do menu (limitação aceita na spec 10).
 */

import type { CadEntityId, CadPoint } from '$lib/types/cad';
import { camadaCorrente } from './camada';
import { lerOpcao, type ComandoEmCurso, type ContextoDeComando, type Opcao } from './motor';
import { formatosDePonto, lerPonto } from './pontos';

const DESFAZER: Opcao = { pt: 'Desfazer', abreviacaoPt: 'D', en: 'Undo', abreviacaoEn: 'U' };
const FECHAR: Opcao = { pt: 'Fechar', abreviacaoPt: 'F', en: 'Close', abreviacaoEn: 'C' };

/** Uma reta criada por este LINE, e a altura da pilha logo depois dela. */
type Segmento = { id: CadEntityId; ate: CadPoint; profundidade: number };

export function linha({ documento, aposEditar }: ContextoDeComando): ComandoEmCurso {
	const camada = camadaCorrente(documento);
	let primeiro: CadPoint | null = null;
	const segmentos: Segmento[] = [];

	const ultimo = (): CadPoint | null => segmentos.at(-1)?.ate ?? primeiro;
	const opcoes = (): Opcao[] => (segmentos.length >= 2 ? [FECHAR, DESFAZER] : [DESFAZER]);

	function criar(de: CadPoint, ate: CadPoint): void {
		const id = documento.drawLine(camada, de, ate);
		segmentos.push({ id, ate, profundidade: documento.getHistory().undoDepth });
		aposEditar();
	}

	/**
	 * Tira a última reta. Se a pilha ainda está como o LINE a deixou, desfaz o
	 * passo; se outra ação entrou depois, apaga a reta, para não desfazer o alheio.
	 */
	function desfazerUltima(): void {
		const segmento = segmentos.at(-1);
		if (segmento == null) return;
		if (documento.getHistory().undoDepth === segmento.profundidade) documento.undo();
		else documento.eraseEntity(segmento.id);
		segmentos.pop();
		aposEditar();
	}

	return {
		querPonto: () => true,

		prompt() {
			if (primeiro == null) return 'Especifique o primeiro ponto:';
			const lista = opcoes()
				.map((o) => o.pt)
				.join('/');
			return `Especifique o próximo ponto ou [${lista}]:`;
		},

		responder(texto) {
			if (texto.trim() === '') return { tipo: 'fim' };

			const anterior = ultimo();
			if (anterior != null) {
				const opcao = lerOpcao(texto, opcoes());
				if (opcao === DESFAZER) {
					if (segmentos.length === 0) {
						return { tipo: 'invalida', mensagem: 'Não há reta para desfazer.' };
					}
					desfazerUltima();
					return { tipo: 'continua' };
				}
				if (opcao === FECHAR && primeiro != null) {
					criar(anterior, primeiro);
					return { tipo: 'fim' };
				}
			}

			const leitura = lerPonto(texto, anterior);
			if (leitura.tipo === 'sem-anterior') {
				return {
					tipo: 'invalida',
					mensagem: 'Não há ponto anterior para a coordenada relativa.'
				};
			}
			if (leitura.tipo === 'invalido') {
				const mensagem =
					anterior == null
						? `Esperava um ponto: ${formatosDePonto()}.`
						: `Esperava um ponto (${formatosDePonto()}) ou uma opção: ${opcoes()
								.map((o) => o.pt)
								.join(', ')}.`;
				return { tipo: 'invalida', mensagem };
			}

			if (anterior == null) primeiro = leitura.ponto;
			else criar(anterior, leitura.ponto);
			return { tipo: 'continua' };
		},

		encerrar() {
			// As retas já criadas ficam (spec 08, C7).
		}
	};
}
