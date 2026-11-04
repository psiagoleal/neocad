// Caminho relativo: src/lib/command-line/apagar.ts

/**
 * \file src/lib/command-line/apagar.ts
 * \brief O comando ERASE / APAGAR / E (spec 10, E1 e E2).
 * \author Iago Leal
 * \date 2026-11-02
 *
 * Com seleção, apaga já; sem ela, pede objetos e apaga o que estiver selecionado
 * quando o usuário confirma o vazio. Tudo num só passo de desfazer.
 */

import type { ComandoEmCurso, ContextoDeComando, Resposta } from './motor';

export function apagar({ documento, aposEditar, selecao }: ContextoDeComando): ComandoEmCurso {
	/** Apaga a seleção, se houver, e encerra. */
	function apagarSelecao(): Resposta {
		const ids = selecao.ids();
		if (ids.length > 0) {
			documento.eraseEntities(ids);
			selecao.esvaziar();
			aposEditar();
		}
		return { tipo: 'fim' };
	}

	return {
		iniciar() {
			return selecao.ids().length > 0 ? apagarSelecao() : { tipo: 'continua' };
		},

		prompt: () => 'Selecione objetos:',

		responder(texto) {
			if (texto.trim() === '') return apagarSelecao();
			return {
				tipo: 'invalida',
				mensagem: 'Selecione objetos na tela; Enter apaga o que estiver selecionado.'
			};
		},

		encerrar() {
			// Nada a desfazer: só apaga quando confirma.
		}
	};
}
