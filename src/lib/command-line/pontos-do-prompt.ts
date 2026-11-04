// Caminho relativo: src/lib/command-line/pontos-do-prompt.ts

/**
 * \file src/lib/command-line/pontos-do-prompt.ts
 * \brief Lê um ponto num prompt e diz o que se esperava quando não é (spec 10, D3).
 * \author Iago Leal
 * \date 2026-11-02
 */

import type { CadPoint } from '$lib/types/cad';
import type { Resposta } from './motor';
import { formatosDePonto, lerPonto } from './pontos';

/** O ponto lido, ou a resposta inválida com o que se esperava. */
export function pontoOuErro(
	texto: string,
	anterior: CadPoint | null,
	opcoes: string[] = []
): CadPoint | Extract<Resposta, { tipo: 'invalida' }> {
	const leitura = lerPonto(texto, anterior);
	if (leitura.tipo === 'ponto') return leitura.ponto;
	if (leitura.tipo === 'sem-anterior') {
		return { tipo: 'invalida', mensagem: 'Não há ponto anterior para a coordenada relativa.' };
	}
	return {
		tipo: 'invalida',
		mensagem:
			opcoes.length === 0
				? `Esperava um ponto: ${formatosDePonto()}.`
				: `Esperava um ponto (${formatosDePonto()}) ou uma opção: ${opcoes.join(', ')}.`
	};
}

export function ehPonto(valor: CadPoint | Resposta): valor is CadPoint {
	return 'x' in valor;
}
